//! A small ordered keyboard map.
//!
//! Applications list their chords once and reuse the same commands in menus, the
//! palette, and the keyboard menu. Bindings are checked in order, so put longer chords
//! before shorter ones (`Ctrl+Shift+P` before `Ctrl+P`). A match consumes the key, so
//! widgets and text fields never see it.

use egui::{Context, Key, Modifiers};

#[derive(Clone, Debug)]
struct Binding<C> {
    modifiers: Modifiers,
    key: Key,
    command: C,
}

/// Ordered shortcuts that resolve to application commands.
///
/// ```
/// # use egui_workbench::app::Keymap;
/// use egui_workbench::egui::{Key, Modifiers};
/// let map = Keymap::new()
///     .bind(Modifiers::COMMAND, Key::S, "save")
///     .bind(Modifiers::COMMAND, Key::N, "new");
/// assert_eq!(map.len(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct Keymap<C> {
    bindings: Vec<Binding<C>>,
}

impl<C> Default for Keymap<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Keymap<C> {
    /// An empty map.
    pub fn new() -> Self {
        Self { bindings: Vec::new() }
    }

    /// Add a binding. Later bindings with the same chord are unreachable.
    pub fn bind(mut self, modifiers: Modifiers, key: Key, command: C) -> Self {
        self.bindings.push(Binding {
            modifiers,
            key,
            command,
        });
        self
    }

    /// Add a binding in place.
    pub fn insert(&mut self, modifiers: Modifiers, key: Key, command: C) {
        self.bindings.push(Binding {
            modifiers,
            key,
            command,
        });
    }

    /// Number of bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether no binding is registered.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Whether a chord is registered.
    pub fn contains(&self, modifiers: Modifiers, key: Key) -> bool {
        self.bindings
            .iter()
            .any(|binding| binding.modifiers == modifiers && binding.key == key)
    }

    /// Remove every binding.
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    /// The command for a key pressed this frame, consuming the key.
    ///
    /// Call this only while no modal is open, so shortcuts do not fire underneath one.
    pub fn command(&mut self, ctx: &Context) -> Option<&C> {
        for binding in &self.bindings {
            if ctx.input_mut(|input| input.consume_key(binding.modifiers, binding.key)) {
                return Some(&binding.command);
            }
        }
        None
    }

    /// The command for a key pressed this frame, cloned.
    pub fn command_cloned(&mut self, ctx: &Context) -> Option<C>
    where
        C: Clone,
    {
        self.command(ctx).cloned()
    }
}

#[cfg(test)]
mod tests {
    use egui::{Event, RawInput};

    use super::*;

    fn press(ctx: &Context, modifiers: Modifiers, key: Key) -> bool {
        let event = Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let input = RawInput {
            events: vec![event],
            ..RawInput::default()
        };
        let mut hit = false;
        let mut map = Keymap::new()
            .bind(Modifiers::COMMAND | Modifiers::SHIFT, Key::P, "palette")
            .bind(Modifiers::COMMAND, Key::P, "quick-open")
            .bind(Modifiers::NONE, Key::F2, "rename");
        let mut output = ctx.run_ui(input, |ui| {
            hit = map.command(ui.ctx()).is_some_and(|command| match *command {
                "palette" => modifiers == (Modifiers::COMMAND | Modifiers::SHIFT),
                "quick-open" => modifiers == Modifiers::COMMAND,
                "rename" => modifiers == Modifiers::NONE,
                _ => false,
            });
        });
        output.textures_delta.clear();
        hit
    }

    #[test]
    fn longer_chords_win_when_listed_first() {
        let ctx = Context::default();
        assert!(press(&ctx, Modifiers::COMMAND | Modifiers::SHIFT, Key::P));
    }

    #[test]
    fn plain_chords_match() {
        let ctx = Context::default();
        assert!(press(&ctx, Modifiers::COMMAND, Key::P));
        assert!(press(&ctx, Modifiers::NONE, Key::F2));
    }

    #[test]
    fn unbound_keys_are_left_for_widgets() {
        let ctx = Context::default();
        assert!(!press(&ctx, Modifiers::NONE, Key::A));
    }

    #[test]
    fn duplicate_chords_keep_the_first_binding() {
        let map = Keymap::new()
            .bind(Modifiers::COMMAND, Key::S, 1)
            .bind(Modifiers::COMMAND, Key::S, 2);
        assert_eq!(map.len(), 2);
        assert!(map.contains(Modifiers::COMMAND, Key::S));
    }
}
