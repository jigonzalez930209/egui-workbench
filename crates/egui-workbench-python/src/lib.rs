//! Python bindings for egui-workbench.
//!
//! The native module is intentionally small: it owns a `HostSpec`, a data set map, an
//! action queue, and the eframe window. The Pythonic layer in `python/egui_workbench`
//! builds the spec, dispatches events, and exposes the decorators and widgets.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use arrow::array::{RecordBatch, RecordBatchReader, StructArray, make_array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ffi::{FFI_ArrowArray, FFI_ArrowSchema};
use arrow::ffi_stream::{FFI_ArrowArrayStream, RecordBatchStreamReader};
use egui_workbench::data::{Column, Dataset, dataset_from_batch};
use egui_workbench_host::{ActionQueue, Host, HostAction, HostEvent, HostSpec, Patch, ToastSpec};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyCapsule, PyCapsuleMethods, PyDict, PyTuple};

/// The native workbench. Every method takes `&self`: Python handlers run while a method
/// is on the stack, and they may call back into `queue_patch`, `post`, or `wake`.
#[pyclass(name = "NativeWorkbench")]
struct NativeWorkbench {
    spec_json: Mutex<String>,
    actions: ActionQueue,
    posts: Arc<Mutex<VecDeque<Py<PyAny>>>>,
    datasets: Mutex<BTreeMap<String, Dataset>>,
    headless: Mutex<Option<Host>>,
}

#[pymethods]
impl NativeWorkbench {
    #[new]
    fn new(spec_json: &str) -> PyResult<Self> {
        HostSpec::from_json(spec_json).map_err(PyValueError::new_err)?;
        Ok(Self {
            spec_json: Mutex::new(spec_json.to_owned()),
            actions: ActionQueue::default(),
            posts: Arc::new(Mutex::new(VecDeque::new())),
            datasets: Mutex::new(BTreeMap::new()),
            headless: Mutex::new(None),
        })
    }

    /// The current spec as JSON. When a headless host exists, this includes the patches
    /// its last frame applied.
    fn spec_json(&self) -> String {
        {
            let headless = self.headless.lock().unwrap_or_else(|error| error.into_inner());
            if let Some(host) = headless.as_ref()
                && let Ok(json) = serde_json::to_string(host.json())
            {
                return json;
            }
        }
        self.spec_json.lock().unwrap_or_else(|error| error.into_inner()).clone()
    }

    /// Apply a patch to the stored spec (and to the headless host, when it exists).
    fn apply_patch(&self, patch_json: &str) -> PyResult<()> {
        let patch: Patch =
            serde_json::from_str(patch_json).map_err(|error| PyValueError::new_err(error.to_string()))?;
        {
            let mut text = self.spec_json.lock().unwrap_or_else(|error| error.into_inner());
            let mut value: serde_json::Value =
                serde_json::from_str(&text).map_err(|error| PyValueError::new_err(error.to_string()))?;
            patch
                .apply(&mut value)
                .map_err(|error| PyValueError::new_err(error.to_string()))?;
            let spec: HostSpec =
                serde_json::from_value(value).map_err(|error| PyValueError::new_err(error.to_string()))?;
            *text = serde_json::to_string(&spec).map_err(|error| PyValueError::new_err(error.to_string()))?;
        }
        let mut headless = self.headless.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(host) = headless.as_mut() {
            host.apply(&patch)
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Queue a patch for the next frame. Safe from an event handler.
    fn queue_patch(&self, patch_json: &str) -> PyResult<()> {
        let patch: Patch =
            serde_json::from_str(patch_json).map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.actions.push(HostAction::Patch(patch));
        Ok(())
    }

    /// Queue a toast for the next frame.
    #[pyo3(signature = (kind, title, description=None))]
    fn queue_toast(&self, kind: &str, title: &str, description: Option<String>) {
        self.actions.push(HostAction::Toast(ToastSpec {
            kind: kind.to_owned(),
            title: title.to_owned(),
            description,
        }));
    }

    /// Ask the window to redraw.
    fn wake(&self) {
        self.actions.push(HostAction::Repaint);
    }

    /// Ask the window to close.
    fn close(&self) {
        self.actions.push(HostAction::Close);
    }

    /// Change the window title on the next frame.
    fn set_title(&self, title: &str) {
        self.actions.push(HostAction::SetTitle(title.to_owned()));
    }

    /// Run a Python callable on the UI thread next frame. Safe from any thread.
    fn post(&self, callback: Py<PyAny>) {
        {
            let mut posts = self.posts.lock().unwrap_or_else(|error| error.into_inner());
            posts.push_back(callback);
        }
        self.wake();
    }

    /// Register a dataset from a mapping of column name to either `(dtype, bytes)` as
    /// produced by the Python layer from a NumPy array, or a Python sequence. Returns the
    /// row count.
    fn set_dataset(&self, name: &str, columns: &Bound<'_, PyDict>) -> PyResult<usize> {
        let mut built = Vec::with_capacity(columns.len());
        for (key, value) in columns.iter() {
            let column = key.extract::<String>()?;
            built.push(column_from_object(&column, &value)?);
        }
        let dataset = Dataset::from_columns(built);
        let rows = dataset.row_count();
        let mut datasets = self.datasets.lock().unwrap_or_else(|error| error.into_inner());
        datasets.insert(name.to_owned(), dataset);
        Ok(rows)
    }

    /// Register a dataset from anything implementing the Arrow PyCapsule interface
    /// (`__arrow_c_stream__`, as pandas, Polars, and PyArrow tables do, or
    /// `__arrow_c_array__`).
    fn set_arrow(&self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<usize> {
        let dataset = arrow_dataset(value)?;
        let rows = dataset.row_count();
        let mut datasets = self.datasets.lock().unwrap_or_else(|error| error.into_inner());
        datasets.insert(name.to_owned(), dataset);
        Ok(rows)
    }

    /// Row count of a registered dataset.
    fn dataset_rows(&self, name: &str) -> Option<usize> {
        self.datasets
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(name)
            .map(Dataset::row_count)
    }

    /// Run a native window. `dispatcher` receives one JSON event string at a time.
    fn run(&self, py: Python<'_>, dispatcher: Py<PyAny>) -> PyResult<()> {
        let spec_json = self.spec_json();
        let datasets: Vec<(String, Dataset)> = self
            .datasets
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .iter()
            .map(|(name, dataset)| (name.clone(), dataset.clone()))
            .collect();
        let actions = self.actions.clone();
        let posts = Arc::clone(&self.posts);
        py.detach(move || {
            let spec = HostSpec::from_json(&spec_json).map_err(|error| error.to_string())?;
            let mut host = Host::with_actions(spec, actions).map_err(|error| error.to_string())?;
            for (name, dataset) in datasets {
                host.set_dataset(name, dataset);
            }
            host.run_with_frame(
                move |event| {
                    Python::attach(|py| dispatch(py, dispatcher.bind(py), event));
                },
                move || {
                    Python::attach(|py| drain_posts(py, &posts));
                },
            )
            .map_err(|error| error.to_string())
        })
        .map_err(PyRuntimeError::new_err)
    }

    /// Render `frames` headless frames and return the JSON events they produced.
    ///
    /// This is the conformance and test path: no window, no display server, the same
    /// tree, patches, actions, and events as a native run.
    #[pyo3(signature = (dispatcher, frames=1))]
    fn step(&self, py: Python<'_>, dispatcher: &Bound<'_, PyAny>, frames: usize) -> PyResult<Vec<String>> {
        let mut events = Vec::new();
        for _ in 0..frames.max(1) {
            drain_posts(py, &self.posts);
            let produced = {
                let mut headless = self.headless.lock().unwrap_or_else(|error| error.into_inner());
                if headless.is_none() {
                    *headless = Some(self.build_host()?);
                }
                headless.as_mut().map(Host::frame).unwrap_or_default()
            };
            for event in produced {
                dispatch(py, dispatcher, &event);
                if let Ok(json) = serde_json::to_string(&event) {
                    events.push(json);
                }
            }
        }
        Ok(events)
    }

    /// The JSON Schema text for `HostSpec`.
    #[staticmethod]
    fn schema() -> String {
        serde_json::to_string_pretty(&HostSpec::schema()).unwrap_or_default()
    }

    /// The package version.
    #[staticmethod]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl NativeWorkbench {
    fn build_host(&self) -> PyResult<Host> {
        let text = self.spec_json.lock().unwrap_or_else(|error| error.into_inner()).clone();
        let spec = HostSpec::from_json(&text).map_err(PyRuntimeError::new_err)?;
        let mut host = Host::with_actions(spec, self.actions.clone())
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let datasets = self.datasets.lock().unwrap_or_else(|error| error.into_inner());
        for (name, dataset) in datasets.iter() {
            host.set_dataset(name.clone(), dataset.clone());
        }
        Ok(host)
    }
}

fn dispatch(py: Python<'_>, dispatcher: &Bound<'_, PyAny>, event: &HostEvent) {
    let Ok(json) = serde_json::to_string(event) else {
        return;
    };
    if let Err(error) = dispatcher.call1((json,)) {
        error.print(py);
    }
}

fn drain_posts(py: Python<'_>, posts: &Arc<Mutex<VecDeque<Py<PyAny>>>>) {
    loop {
        let callback = {
            let mut posts = posts.lock().unwrap_or_else(|error| error.into_inner());
            posts.pop_front()
        };
        let Some(callback) = callback else {
            break;
        };
        if let Err(error) = callback.call0(py) {
            error.print(py);
        }
    }
}

fn column_from_object(name: &str, value: &Bound<'_, PyAny>) -> PyResult<Column> {
    if let Ok((dtype, bytes)) = value.extract::<(String, Bound<'_, PyBytes>)>() {
        return encoded_column(name, &dtype, bytes.as_bytes());
    }
    if let Ok(values) = value.extract::<Vec<Option<bool>>>() {
        return Ok(optional_column(values, |values| Column::bools(name, values)));
    }
    if let Ok(values) = value.extract::<Vec<Option<i64>>>() {
        return Ok(optional_column(values, |values| Column::ints(name, values)));
    }
    if let Ok(values) = value.extract::<Vec<Option<f64>>>() {
        return Ok(optional_column(values, |values| Column::floats(name, values)));
    }
    if let Ok(values) = value.extract::<Vec<Option<String>>>() {
        return Ok(optional_column(values, |values| Column::text(name, values)));
    }
    Err(PyValueError::new_err(format!(
        "column `{name}` is not an encoded array, a numeric sequence, a boolean sequence, or a string sequence"
    )))
}

/// Decode the little-endian NumPy dtype string and the raw bytes from `tobytes()`.
fn encoded_column(name: &str, dtype: &str, bytes: &[u8]) -> PyResult<Column> {
    let dtype = dtype.trim_start_matches(['<', '=', '|']);
    if let Some(dtype) = dtype.strip_prefix('>') {
        return Err(PyValueError::new_err(format!(
            "big-endian dtype `>{dtype}` is not supported; call `astype` with a native dtype first"
        )));
    }
    let width = match dtype {
        "f8" | "i8" | "u8" => 8,
        "f4" | "i4" | "u4" => 4,
        "i2" | "u2" => 2,
        _ => 1,
    };
    // `chunks_exact` would silently drop a trailing partial value.
    if !bytes.len().is_multiple_of(width) {
        return Err(PyValueError::new_err(format!(
            "column `{name}` has {} bytes, which is not a multiple of the {width}-byte `{dtype}` width",
            bytes.len()
        )));
    }
    let column = match dtype {
        "f8" => Column::floats(name, chunks::<8>(bytes).map(f64::from_le_bytes).collect::<Vec<_>>()),
        "f4" => Column::floats(
            name,
            chunks::<4>(bytes)
                .map(|chunk| f64::from(f32::from_le_bytes(chunk)))
                .collect::<Vec<_>>(),
        ),
        "i8" => Column::ints(name, chunks::<8>(bytes).map(i64::from_le_bytes).collect::<Vec<_>>()),
        "i4" => Column::ints(
            name,
            chunks::<4>(bytes)
                .map(|chunk| i64::from(i32::from_le_bytes(chunk)))
                .collect::<Vec<_>>(),
        ),
        "i2" => Column::ints(
            name,
            chunks::<2>(bytes)
                .map(|chunk| i64::from(i16::from_le_bytes(chunk)))
                .collect::<Vec<_>>(),
        ),
        "i1" => Column::ints(
            name,
            bytes.iter().map(|byte| i64::from(*byte as i8)).collect::<Vec<_>>(),
        ),
        "u8" => Column::ints(
            name,
            chunks::<8>(bytes)
                .map(|chunk| u64::from_le_bytes(chunk) as i64)
                .collect::<Vec<_>>(),
        ),
        "u4" => Column::ints(
            name,
            chunks::<4>(bytes)
                .map(|chunk| i64::from(u32::from_le_bytes(chunk)))
                .collect::<Vec<_>>(),
        ),
        "u2" => Column::ints(
            name,
            chunks::<2>(bytes)
                .map(|chunk| i64::from(u16::from_le_bytes(chunk)))
                .collect::<Vec<_>>(),
        ),
        "u1" => Column::ints(name, bytes.iter().map(|byte| i64::from(*byte)).collect::<Vec<_>>()),
        "b1" | "?" => Column::bools(name, bytes.iter().map(|byte| *byte != 0).collect::<Vec<_>>()),
        other => {
            return Err(PyValueError::new_err(format!(
                "unsupported dtype `{other}`; expected f8, f4, i8, i4, i2, i1, u8, u4, u2, u1, or b1"
            )));
        }
    };
    Ok(column)
}

fn chunks<const N: usize>(bytes: &[u8]) -> impl Iterator<Item = [u8; N]> + '_ {
    bytes.chunks_exact(N).map(move |chunk| {
        let mut buffer = [0u8; N];
        buffer.copy_from_slice(chunk);
        buffer
    })
}

fn optional_column<T: Default>(values: Vec<Option<T>>, make: impl FnOnce(Vec<T>) -> Column) -> Column {
    let validity: Vec<bool> = values.iter().map(Option::is_some).collect();
    let has_nulls = validity.iter().any(|valid| !valid);
    let values: Vec<T> = values.into_iter().map(Option::unwrap_or_default).collect();
    let column = make(values);
    if has_nulls {
        column.with_validity(validity)
    } else {
        column
    }
}

fn arrow_dataset(value: &Bound<'_, PyAny>) -> PyResult<Dataset> {
    if let Ok(method) = value.getattr("__arrow_c_stream__") {
        let capsule = method.call0()?;
        let capsule = capsule
            .cast::<PyCapsule>()
            .map_err(|_| PyValueError::new_err("__arrow_c_stream__ did not return a capsule"))?;
        let pointer = capsule
            .pointer_checked(Some(c"arrow_array_stream"))
            .map_err(|error| PyValueError::new_err(format!("invalid Arrow stream capsule: {error}")))?;
        let mut pointer = pointer.cast::<FFI_ArrowArrayStream>();
        // SAFETY: the capsule owns a valid ArrowArrayStream; ownership moves into the reader.
        let reader = unsafe { RecordBatchStreamReader::from_raw(pointer.as_mut()) }
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let schema = reader.schema();
        let batches = reader
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let batch = if batches.len() == 1 {
            batches
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("empty Arrow stream"))?
        } else {
            arrow::compute::concat_batches(&schema, &batches)
                .map_err(|error| PyValueError::new_err(error.to_string()))?
        };
        return dataset_from_batch(&batch).map_err(|error| PyValueError::new_err(error.to_string()));
    }
    let method = value
        .getattr("__arrow_c_array__")
        .map_err(|_| PyValueError::new_err("object has neither __arrow_c_stream__ nor __arrow_c_array__"))?;
    let pair = method.call0()?;
    let pair = pair
        .cast::<PyTuple>()
        .map_err(|_| PyValueError::new_err("__arrow_c_array__ did not return a tuple"))?;
    if pair.len() != 2 {
        return Err(PyValueError::new_err("__arrow_c_array__ must return (schema, array)"));
    }
    let schema_capsule = pair
        .get_item(0)?
        .cast::<PyCapsule>()
        .map_err(|_| PyValueError::new_err("Arrow schema is not a capsule"))?
        .clone();
    let array_capsule = pair
        .get_item(1)?
        .cast::<PyCapsule>()
        .map_err(|_| PyValueError::new_err("Arrow array is not a capsule"))?
        .clone();
    let schema_pointer = schema_capsule
        .pointer_checked(Some(c"arrow_schema"))
        .map_err(|error| PyValueError::new_err(format!("invalid Arrow schema capsule: {error}")))?;
    let array_pointer = array_capsule
        .pointer_checked(Some(c"arrow_array"))
        .map_err(|error| PyValueError::new_err(format!("invalid Arrow array capsule: {error}")))?;
    // SAFETY: both capsules follow the Arrow C data interface and remain alive until after
    // `from_ffi` has imported the buffers. Moving the structs out and replacing them with
    // empty structs transfers release ownership exactly once.
    let (array, schema) = unsafe {
        let array = array_pointer.cast::<FFI_ArrowArray>().as_ptr();
        let schema = schema_pointer.cast::<FFI_ArrowSchema>().as_ptr();
        let array_value = std::ptr::read(array);
        let schema_value = std::ptr::read(schema);
        std::ptr::write(array, FFI_ArrowArray::empty());
        std::ptr::write(schema, FFI_ArrowSchema::empty());
        (array_value, schema_value)
    };
    let data =
        unsafe { arrow::ffi::from_ffi(array, &schema) }.map_err(|error| PyValueError::new_err(error.to_string()))?;
    let batch = if matches!(data.data_type(), DataType::Struct(_)) {
        let array = StructArray::from(data);
        let fields = array
            .fields()
            .iter()
            .map(|field| field.as_ref().clone())
            .collect::<Vec<_>>();
        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, array.columns().to_vec())
            .map_err(|error| PyValueError::new_err(error.to_string()))?
    } else {
        let array = make_array(data);
        let field = Field::new("value", array.data_type().clone(), true);
        RecordBatch::try_new(Arc::new(Schema::new(vec![field])), vec![array])
            .map_err(|error| PyValueError::new_err(error.to_string()))?
    };
    dataset_from_batch(&batch).map_err(|error| PyValueError::new_err(error.to_string()))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<NativeWorkbench>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
