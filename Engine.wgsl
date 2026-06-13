// الهيكل الخاص بالبيانات الموحدة (Camera Uniform)
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// مدخلات الـ Vertex Shader للمكعب والأرضية
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>, // سنحتاجها لحسابات خطوط الشبكة
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = model.position;
    // تحويل الإحداثيات من الفضاء المحلي إلى فضاء الكاميرا الشاشة
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // إذا كان اللون القادم (0.0, 0.0, 0.0) فهذا يعني أنها الأرضية الشبكية (Procedural Grid)
    if (in.color.r == 0.0 && in.color.g == 0.0 && in.color.b == 0.0) {
        // حسابات رسم خطوط الشبكة بناءً على موقع العالم ثلاثي الأبعاد
        let coord = in.world_pos.xz * 2.0; 
        let grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
        let line = min(grid.x, grid.y);
        let color_intensity = 1.0 - min(line, 1.0);
        
        // إذا كان القرب من الخط كبيراً نرسم خط شبكة رمادي، وإلا نجعل الأرضية شفافة/داكنة كالمحركات الاحترافية
        if (color_intensity > 0.1) {
            return vec4<f32>(0.3, 0.3, 0.3, 1.0); // لون خطوط الشبكة (Blender Style)
        }
        return vec4<f32>(0.1, 0.1, 0.1, 1.0); // لون الخلفية الداكن للأرضية
    }

    // هنا رندرة لون المكعب العادي (شكل Blender الأصلي الرمادي الجميل مع بعض الإضاءة البسيطة)
    let ambient = vec3<f32>(0.2, 0.2, 0.2);
    let final_color = in.color + ambient;
    return vec4<f32>(final_color, 1.0);
}