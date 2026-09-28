class_name Tcc2Light extends Tcc2Node

const DIRECTION_LENGTH := 0.5

@export var color: Color = Color.WHITE:
	set(value):
		color = value
		_rebuild_preview()

@export var intensity: float = 1.0

var _preview: Sprite3D
var _direction: MeshInstance3D
var _light: Light3D

func _ready() -> void:
	super()

	_preview = _add_icon(preload("res://icons/light.svg"))

	_direction = MeshInstance3D.new()
	add_child(_direction, false, Node.INTERNAL_MODE_FRONT)

	_rebuild_preview()

func set_selected(value: bool) -> void:
	if _preview != null:
		_set_icon_selected(_preview, value, color)
		_direction.material_override = _selected_gizmo_material() if value else null

func _rebuild_preview() -> void:
	if _preview == null:
		return

	_preview.modulate = color
	_direction.mesh = _gizmo_mesh()

	if _light != null:
		remove_child(_light)
		_light.queue_free()
		_light = null

	_light = _light_preview()

	if _light == null:
		return

	_light.light_color = color
	_light.light_energy = 1.0

	add_child(_light, false, Node.INTERNAL_MODE_FRONT)

func _gizmo_mesh() -> Mesh:
	return null

func _light_preview() -> Light3D:
	return null
