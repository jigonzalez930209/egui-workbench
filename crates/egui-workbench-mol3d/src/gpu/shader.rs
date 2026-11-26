//! WGSL for analytic sphere and cylinder impostors and the ID-buffer pass.

/// Sphere and cylinder display plus ID shaders.
pub(crate) const WGSL: &str = r#"
struct Uniform {
    eye: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    forward: vec4<f32>,
    params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Uniform;

const CORNERS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

fn ray_from_ndc(ndc: vec2<f32>) -> vec3<f32> {
    let tan_half = camera.params.x;
    let aspect = camera.params.y;
    return normalize(
        camera.right.xyz * (ndc.x * tan_half * aspect)
            + camera.up.xyz * (ndc.y * tan_half)
            + camera.forward.xyz
    );
}

fn project_point(point: vec3<f32>) -> vec3<f32> {
    let relative = point - camera.eye.xyz;
    let depth = dot(relative, camera.forward.xyz);
    let tan_half = camera.params.x;
    let aspect = camera.params.y;
    let x = dot(relative, camera.right.xyz) / (depth * tan_half * aspect);
    let y = dot(relative, camera.up.xyz) / (depth * tan_half);
    return vec3<f32>(x, y, depth);
}

struct SphereInstance {
    @location(0) center: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
    @location(3) pick: u32,
};

struct SphereOut {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
    @location(1) center: vec3<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
    @location(4) @interpolate(flat) pick: u32,
};

@vertex
fn vs_sphere(@builtin(vertex_index) vertex: u32, instance: SphereInstance) -> SphereOut {
    var out: SphereOut;
    let corner = CORNERS[vertex] * instance.radius * 1.35;
    let world = instance.center + camera.right.xyz * corner.x + camera.up.xyz * corner.y;
    let relative = world - camera.eye.xyz;
    let depth = dot(relative, camera.forward.xyz);
    let tan_half = camera.params.x;
    let aspect = camera.params.y;
    let x = dot(relative, camera.right.xyz) / (depth * tan_half * aspect);
    let y = dot(relative, camera.up.xyz) / (depth * tan_half);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.ndc = vec2<f32>(x, y);
    out.center = instance.center;
    out.radius = instance.radius;
    out.color = instance.color;
    out.pick = instance.pick;
    return out;
}

fn sphere_hit(ndc: vec2<f32>, center: vec3<f32>, radius: f32) -> f32 {
    let ray = ray_from_ndc(ndc);
    let oc = camera.eye.xyz - center;
    let b = dot(oc, ray);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - c;
    if (discriminant < 0.0) { return -1.0; }
    let t = -b - sqrt(discriminant);
    if (t < 0.0) { return -1.0; }
    return t;
}

fn shade(color: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    let light = normalize(vec3<f32>(0.35, 0.7, 0.9));
    let diffuse = max(dot(normal, light), 0.18);
    let base = pow(color.rgb, vec3<f32>(2.2));
    return vec4<f32>(base * (0.42 + 0.58 * diffuse), color.a);
}

struct ColorOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_sphere(in: SphereOut) -> ColorOut {
    let t = sphere_hit(in.ndc, in.center, in.radius);
    if (t < 0.0) { discard; }
    let ray = ray_from_ndc(in.ndc);
    let hit = camera.eye.xyz + ray * t;
    let normal = normalize(hit - in.center);
    var out: ColorOut;
    out.color = shade(in.color, normal);
    out.depth = clamp(t * 0.001, 0.0, 0.999999);
    return out;
}

struct PickOut {
    @location(0) id: u32,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_sphere_id(in: SphereOut) -> PickOut {
    let t = sphere_hit(in.ndc, in.center, in.radius);
    if (t < 0.0) { discard; }
    var out: PickOut;
    out.id = in.pick;
    out.depth = clamp(t * 0.001, 0.0, 0.999999);
    return out;
}

struct BondInstance {
    @location(0) a: vec3<f32>,
    @location(1) b: vec3<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
    @location(4) pick: u32,
};

struct BondOut {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
    @location(1) a: vec3<f32>,
    @location(2) b: vec3<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
    @location(5) @interpolate(flat) pick: u32,
};

@vertex
fn vs_bond(@builtin(vertex_index) vertex: u32, instance: BondInstance) -> BondOut {
    var out: BondOut;
    let pa = project_point(instance.a);
    let pb = project_point(instance.b);
    if (pa.z <= 0.01 || pb.z <= 0.01) {
        out.position = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        out.ndc = vec2<f32>(2.0, 2.0);
        out.a = instance.a;
        out.b = instance.b;
        out.radius = instance.radius;
        out.color = instance.color;
        out.pick = instance.pick;
        return out;
    }
    let corner = CORNERS[vertex];
    let t = select(0.0, 1.0, corner.x > 0.0);
    let base = mix(pa, pb, t);
    var direction = normalize(pb.xy - pa.xy);
    if (!(abs(direction.x) + abs(direction.y) > 0.0001)) {
        direction = vec2<f32>(1.0, 0.0);
    }
    let perpendicular = vec2<f32>(-direction.y, direction.x);
    let radius_ndc = instance.radius / (base.z * camera.params.x) * 2.2;
    let position = base.xy + perpendicular * corner.y * radius_ndc;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.ndc = position;
    out.a = instance.a;
    out.b = instance.b;
    out.radius = instance.radius;
    out.color = instance.color;
    out.pick = instance.pick;
    return out;
}

fn capsule_hit(ndc: vec2<f32>, a: vec3<f32>, b: vec3<f32>, radius: f32) -> f32 {
    let ray = ray_from_ndc(ndc);
    let ba = b - a;
    let oa = camera.eye.xyz - a;
    let baba = dot(ba, ba);
    let bard = dot(ba, ray);
    let baoa = dot(ba, oa);
    let rdoa = dot(ray, oa);
    let denominator = baba - bard * bard;
    if (abs(denominator) < 0.000001) { return -1.0; }
    let t_axis = clamp((baoa - rdoa * bard) / denominator, 0.0, 1.0);
    let closest = a + ba * t_axis;
    let t = dot(closest - camera.eye.xyz, ray);
    if (t <= 0.0) { return -1.0; }
    let hit = camera.eye.xyz + ray * t;
    let radial = hit - closest;
    if (dot(radial, radial) > radius * radius) { return -1.0; }
    return t;
}

@fragment
fn fs_bond(in: BondOut) -> ColorOut {
    let t = capsule_hit(in.ndc, in.a, in.b, in.radius);
    if (t < 0.0) { discard; }
    let ray = ray_from_ndc(in.ndc);
    let hit = camera.eye.xyz + ray * t;
    let ba = in.b - in.a;
    let closest = in.a + ba * clamp(dot(hit - in.a, ba) / max(dot(ba, ba), 0.000001), 0.0, 1.0);
    let normal = normalize(hit - closest);
    var out: ColorOut;
    out.color = shade(in.color, normal);
    out.depth = clamp(t * 0.001, 0.0, 0.999999);
    return out;
}

@fragment
fn fs_bond_id(in: BondOut) -> PickOut {
    let t = capsule_hit(in.ndc, in.a, in.b, in.radius);
    if (t < 0.0) { discard; }
    var out: PickOut;
    out.id = in.pick;
    out.depth = clamp(t * 0.001, 0.0, 0.999999);
    return out;
}
"#;

/// Fullscreen triangle that samples the offscreen scene texture.
pub(crate) const QUAD_WGSL: &str = r#"
@group(0) @binding(0) var scene_texture: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;

struct QuadOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_quad(@builtin(vertex_index) vertex: u32) -> QuadOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let position = positions[vertex];
    var out: QuadOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>((position.x + 1.0) * 0.5, 1.0 - (position.y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_quad(in: QuadOut) -> @location(0) vec4<f32> {
    return textureSample(scene_texture, scene_sampler, in.uv);
}
"#;
