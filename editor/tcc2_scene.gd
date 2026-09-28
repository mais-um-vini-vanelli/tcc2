class_name Tcc2Scene extends Tcc2Node

@export_group("transform")
@export_enum("Stepwise") var transform_method: int = 0

@export_group("rasterization")
@export var backface_cull: bool = true
@export var wireframe: bool = false
@export var perspective_correct: bool = true
@export var depth_test: bool = true

@export_group("clipping")
@export var clipping_enabled: bool = true

@export_group("texturing")
@export var mipmaps: bool = true
@export_enum("Nearest", "Bilinear", "Trilinear") var filtering: int = 2

@export_group("shading")
@export_enum("Flat", "Gouraud", "Phong") var shading_frequency: int = 2
@export_enum("BlinnPhong") var reflectance_model: int = 0

@export_group("ray_tracing")
@export_range(0, 16) var max_depth: int = 5
@export var shadows: bool = true
@export var reflections: bool = true
@export var refractions: bool = true

@export_subgroup("acceleration")
@export var bvh: bool = true
@export_enum("Median", "Sah") var bvh_split: int = 1

@export_group("output")
@export_range(1, 4096) var width: int = 800
@export_range(1, 4096) var height: int = 600
@export var background_enabled: bool = true
@export var background: Color = Color.BLACK
@export var exposure: float = 1.0
@export_enum("None", "Reinhard") var tonemap: int = 0
