class_name SelectionOutline extends ColorRect

const WIDTH := 2.0

@export var scene_view: SubViewport
@export var editor_camera: Camera3D

@onready var _mask: SubViewport = $MaskView
@onready var _camera: Camera3D = $MaskView/Camera3D

func _ready() -> void:
	_mask.world_3d = scene_view.find_world_3d()

	_camera.cull_mask = Tcc2Node.SELECTION_LAYER
	_camera.current = true
	
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color.BLACK
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_DISABLED
	_camera.environment = environment
	
	var shader := material as ShaderMaterial
	shader.set_shader_parameter(&"mask", _mask.get_texture())
	shader.set_shader_parameter(&"width", WIDTH)
	shader.set_shader_parameter(&"line_color", Tcc2Node.SELECTION_COLOR)

func set_enabled(value: bool) -> void:
	visible = value
	_mask.render_target_update_mode = (SubViewport.UPDATE_ALWAYS if value
		else SubViewport.UPDATE_DISABLED)

func _process(_delta: float) -> void:
	if not visible:
		return

	if _mask.size != scene_view.size:
		_mask.size = scene_view.size

	(material as ShaderMaterial).set_shader_parameter(&"mask_size", Vector2(_mask.size))

	_camera.global_transform = editor_camera.global_transform
	_camera.keep_aspect = editor_camera.keep_aspect
	_camera.near = editor_camera.near
	_camera.far = editor_camera.far
	_camera.fov = editor_camera.fov
	_camera.size = editor_camera.size
	_camera.projection = editor_camera.projection
