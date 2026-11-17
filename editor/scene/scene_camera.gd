class_name SceneCamera extends SceneObject

const GIZMO_DEPTH := 0.8
const GIZMO_COLOR := Color(ICON_COLOR, 0.7)

enum Kind {PERSPECTIVE, ORTHOGRAPHIC}

class Perspective:
	var camera: SceneCamera

	var fov: float = 60.0:
		set(value):
			fov = value
			camera._rebuild_preview()

	var near: float = 0.1
	var far: float = 100.0

class Orthographic:
	var camera: SceneCamera

	var height: float = 1.0:
		set(value):
			height = value
			camera._rebuild_preview()

	var near: float = 0.1
	var far: float = 100.0

var kind: Kind = Kind.PERSPECTIVE:
	set(value):
		kind = value
		_rebuild_preview()

var perspective := Perspective.new()
var orthographic := Orthographic.new()

var _icon: Sprite3D
var _gizmo: MeshInstance3D

func _init() -> void:
	perspective.camera = self
	orthographic.camera = self

func _ready() -> void:
	super()

	_icon = _add_icon(preload("res://icons/camera.svg"))

	_gizmo = MeshInstance3D.new()
	_gizmo.layers = ICONS_LAYER
	add_child(_gizmo, false, Node.INTERNAL_MODE_FRONT)

	_rebuild_preview()

func model():
	match kind:
		Kind.ORTHOGRAPHIC:
			return orthographic

	return perspective

func set_selected(value: bool) -> void:
	if _icon != null:
		_set_icon_selected(_icon, value, ICON_COLOR)
		_gizmo.material_override = _selected_gizmo_material() if value else null

func aspect() -> float:
	var scene := find_scene()
	if scene == null:
		return 1.0
	return float(scene.width) / float(scene.height)

func _rebuild_preview() -> void:
	if _gizmo == null:
		return

	var is_ortho := kind == Kind.ORTHOGRAPHIC
	var half_h := 0.0

	if is_ortho:
		half_h = orthographic.height * 0.5
	else:
		half_h = tan(deg_to_rad(perspective.fov) * 0.5) * GIZMO_DEPTH

	var far_half := Vector2(half_h * aspect(), half_h)
	var near_half := far_half if is_ortho else Vector2.ZERO

	var lines := GizmoMesh.frustum(near_half, far_half, GIZMO_DEPTH)

	var mat = _gizmo_material()
	mat.albedo_color = GIZMO_COLOR
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA

	var mesh := ImmediateMesh.new()
	GizmoMesh.surface(mesh, Mesh.PRIMITIVE_LINES, mat, lines)

	_gizmo.mesh = mesh
