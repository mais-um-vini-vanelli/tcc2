class_name Tcc2Scene extends Tcc2Node

enum TransformMethod {STEPWISE}
enum Filtering {NEAREST, BILINEAR, TRILINEAR}
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

# ClippingOptions
var clipping_enabled := true

# TextureOptions
var mipmaps := true
var filtering := Filtering.TRILINEAR

# ShadingOptions
var shading_frequency := ShadingFrequency.PHONG
var reflectance_model := ReflectanceModel.BLINN_PHONG

# RayTracingOptions
var max_depth := 5
var shadows := true
var reflections := true
var refractions := true

# RayTracingOptions::acceleration
var bvh := true
var bvh_split := BvhSplit.SAH

# OutputOptions
var width := 800
var height := 600
var background_enabled := true
var background := Color.BLACK
var exposure := 1.0
var tonemap := Tonemap.NONE
