class_name Tcc2Camera extends Tcc2Node

const GIZMO_DEPTH := 0.8

@export var near: float = 0.1
@export var far: float = 100.0

var _icon: Sprite3D
var _preview: MeshInstance3D

func _ready() -> void:
	super()

	_icon = _add_icon(preload("res://icons/camera.svg"))

	_preview = MeshInstance3D.new()
	add_child(_preview, false, Node.INTERNAL_MODE_FRONT)

	_rebuild_preview()

func set_selected(value: bool) -> void:
	if _icon != null:
		_set_icon_selected(_icon, value, Color.WHITE)
		_preview.material_override = _selected_gizmo_material() if value else null

func aspect() -> float:
	var scene := find_scene()
	if scene == null:
		return 1.0
	return float(scene.width) / float(scene.height)

func _rebuild_preview() -> void:
	if _preview == null:
		return

	var half_h := _half_height()
	var half_w := half_h * aspect()

	var far_corners := [
		Vector3(-half_w, half_h, -GIZMO_DEPTH),
		Vector3(half_w, half_h, -GIZMO_DEPTH),
		Vector3(half_w, -half_h, -GIZMO_DEPTH),
		Vector3(-half_w, -half_h, -GIZMO_DEPTH),
	]

	var near_corners: Array[Vector3] = [
		Vector3.ZERO,
		Vector3.ZERO,
		Vector3.ZERO,
		Vector3.ZERO
	]

	if _has_near_plane():
		for i in 4:
			near_corners[i] = Vector3(far_corners[i].x, far_corners[i].y, 0.0)

	var mesh := ImmediateMesh.new()

	mesh.surface_begin(Mesh.PRIMITIVE_LINES, _gizmo_material())

	for i in 4:
		var j := (i + 1) % 4
		_line(mesh, far_corners[i], far_corners[j])
		_line(mesh, near_corners[i], far_corners[i])

		if _has_near_plane():
			_line(mesh, near_corners[i], near_corners[j])

	mesh.surface_end()

	_preview.mesh = mesh

func _half_height() -> float:
	return 0.0

func _has_near_plane() -> bool:
	return false
