class_name Tcc2Scene extends Tcc2Node

enum TransformMethod {STEPWISE}
enum Filtering {NEAREST, BILINEAR, MIPMAP_BILINEAR, TRILINEAR}
enum ShadingFrequency {FLAT, GOURAUD, PHONG}
enum ReflectanceModel {BLINN_PHONG}
enum BvhSplit {MEDIAN, SAH}
enum Tonemap {NONE, REINHARD}

# TransformOptions
var transform_method := TransformMethod.STEPWISE

# RasterizationOptions
var backface_cull := true
var wireframe := false
var perspective_correct := true
var depth_test := true

# ShadingOptions
var shading_frequency := ShadingFrequency.PHONG
var reflectance_model := ReflectanceModel.BLINN_PHONG
var filtering := Filtering.TRILINEAR

# RayTracingOptions
var max_depth := 5
var shadows := true
var reflections := true
var refractions := true

# RayTracingOptions::acceleration
var bvh := true
var bvh_split := BvhSplit.SAH
var bvh_wireframe := 0

# OutputOptions
var width := 800
var height := 600
var background := Color.BLACK
var exposure := 1.0
var tonemap := Tonemap.NONE
