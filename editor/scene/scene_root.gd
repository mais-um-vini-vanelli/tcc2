class_name SceneRoot extends SceneObject

enum TransformMethod {STEPWISE}
enum TextureFilter {NEAREST, BILINEAR, MIPMAP_BILINEAR, TRILINEAR}
enum ReflectanceModel {BLINN_PHONG}
enum Acceleration {BRUTE_FORCE, BVH_MEDIAN, BVH_SAH}
enum Tonemap {NONE, REINHARD}

var assets := SceneAssets.new()

# TransformOptions
var transform_method := TransformMethod.STEPWISE

# RasterizationOptions
var backface_cull := true
var perspective_correct := true
var depth_test := true

# ShadingOptions
var reflectance_model := ReflectanceModel.BLINN_PHONG
var texture_filter := TextureFilter.TRILINEAR

# RayTracingOptions
var max_depth := 5
var shadows := true
var reflections := true
var refractions := true
var acceleration := Acceleration.BVH_SAH

# OutputOptions
var width := 800
var height := 600
var exposure := 0.0
var tonemap := Tonemap.NONE
