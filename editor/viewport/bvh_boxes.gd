class_name BvhBoxes extends Node3D

signal reported(lines: Array[String])

const LEVEL_COLORS: Array[Color] = [
	Color(1.0, 1.00, 1.0),
	Color(1.0, 0.85, 0.2),
	Color(0.3, 1.00, 0.4),
	Color(0.3, 0.80, 1.0),
	Color(0.8, 0.40, 1.0),
	Color(1.0, 0.40, 0.4),
]

const MAX_DEPTH := 64

var scene_view: SubViewport

var _mesh := ImmediateMesh.new()
var _instance: MeshInstance3D
var _active := false
var _depth := 6

var _levels: Array = []
var _warnings: Array[String] = []

func _ready() -> void:
	_instance = MeshInstance3D.new()
	_instance.mesh = _mesh
	_instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_instance.visible = false
	add_child(_instance)

func set_active(value: bool) -> void:
	if _active == value:
		return

	_active = value
	_instance.visible = value

	if value:
		_fetch()
		return

	_levels = []
	_warnings = []
	_mesh.clear_surfaces()

	var none: Array[String] = []
	reported.emit(none)

func handle(event: InputEvent) -> bool:
	var key := event as InputEventKey

	if not _active or key == null or not key.pressed:
		return false

	var step := 0

	match key.keycode:
		KEY_EQUAL, KEY_PLUS, KEY_KP_ADD:
			step = 1
		KEY_MINUS, KEY_KP_SUBTRACT:
			step = -1
		_:
			return false

	var depth := clampi(_depth + step, 1, maxi(_levels.size(), 1))

	if depth != _depth:
		_depth = depth
		_draw_levels()

	return true

func _fetch() -> void:
	_mesh.clear_surfaces()

	var scene := _scene()

	if scene == null:
		return

	_waiting()

	# The build blocks everything: let the toast land on the screen first.
	await get_tree().process_frame
	await get_tree().process_frame

	if not _active:
		return

	_warnings = []

	var bytes := Write.to_tcc2(scene, _warnings)
	_levels = Tcc2Bvh.aabbs(bytes, MAX_DEPTH, _warnings)
	_depth = clampi(_depth, 1, maxi(_levels.size(), 1))

	_instance.global_transform = scene.global_transform

	_draw_levels()

func _draw_levels() -> void:
	_mesh.clear_surfaces()

	for entry in _levels:
		var level: int = entry["level"]

		if level > _depth:
			continue

		var material := StandardMaterial3D.new()
		material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		material.albedo_color = LEVEL_COLORS[(level - 1) % LEVEL_COLORS.size()]

		GizmoMesh.surface(_mesh, Mesh.PRIMITIVE_LINES, material, entry["lines"])

	var lines: Array[String] = _warnings.duplicate()
	lines.push_front("press - or + to change the depth")
	lines.push_front("depth %d of %d" % [_depth, _levels.size()])
	reported.emit(lines)

func _waiting() -> void:
	var lines: Array[String] = ["processando..."]
	reported.emit(lines)

func _scene() -> SceneRoot:
	var found := scene_view.find_children("*", "SceneRoot", true, false)
	return null if found.is_empty() else found[0]
