class_name RayTrace extends Node3D

signal reported(lines: Array[String])

const KIND_COLORS := {
	"primary": Color(1.0, 1.00, 1.0),
	"shadow": Color(1.0, 0.85, 0.2),
	"reflection": Color(0.3, 0.80, 1.0),
	"refraction": Color(0.8, 0.40, 1.0),
}

const NORMAL_COLOR := Color(0.3, 1.00, 0.4)
const NORMAL_LENGTH := 0.15

const DASH_LENGTH := 0.05

var scene_view: SubViewport

var _mesh := ImmediateMesh.new()
var _instance: MeshInstance3D
var _active := false

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

	if not value:
		_mesh.clear_surfaces()

		var none: Array[String] = []
		reported.emit(none)

func trace(x: int, y: int) -> void:
	_mesh.clear_surfaces()

	var scene := _scene()

	if scene == null:
		return

	var waiting: Array[String] = [_pixel(x, y), "processando..."]
	reported.emit(waiting)

	# The render blocks everything: let the toast and the 3d view land first.
	await get_tree().process_frame
	await get_tree().process_frame

	if not _active:
		return

	var lines: Array[String] = []
	var bytes := Write.to_tcc2(scene, lines)
	var traced: Dictionary = Tcc2Rays.trace_pixel(bytes, x, y, lines)

	if traced.is_empty():
		reported.emit(lines)
		return

	_instance.global_transform = scene.global_transform

	var reference := _primary_length(traced["primary"])

	for kind in KIND_COLORS:
		var rays: Dictionary = traced[kind]

		var solid: PackedVector3Array = rays["miss"] if kind == "shadow" else rays["hit"]
		var dotted: PackedVector3Array = rays["hit"] if kind == "shadow" else rays["miss"]

		_surface(solid, KIND_COLORS[kind])
		_surface(_dashed(dotted, reference * DASH_LENGTH), KIND_COLORS[kind])

	_surface(_scale_normals(traced["normals"], reference), NORMAL_COLOR)

	var color: Color = traced["color"]

	var summary: Array[String] = [_pixel(x, y),
		"color: %.3f, %.3f, %.3f" % [color.r, color.g, color.b]]
	summary.append_array(lines)

	reported.emit(summary)

func _surface(points: PackedVector3Array, color: Color) -> void:
	if points.is_empty():
		return

	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_color = color

	GizmoMesh.surface(_mesh, Mesh.PRIMITIVE_LINES, material, points)

func _primary_length(primary: Dictionary) -> float:
	var points: PackedVector3Array = primary["hit"] if primary["miss"].is_empty() else primary["miss"]
	return 0.0 if points.size() < 2 else points[0].distance_to(points[1])

## The normals arrive one unit long, which means nothing on its own.
func _scale_normals(normals: PackedVector3Array, reference: float) -> PackedVector3Array:
	var length := reference * NORMAL_LENGTH
	var scaled := PackedVector3Array()

	for i in range(0, normals.size(), 2):
		scaled.push_back(normals[i])
		scaled.push_back(normals[i] + (normals[i + 1] - normals[i]) * length)

	return scaled

func _dashed(points: PackedVector3Array, length: float) -> PackedVector3Array:
	if length <= 0.0:
		return points

	var dashed := PackedVector3Array()

	for i in range(0, points.size(), 2):
		var from := points[i]
		var to := points[i + 1]
		var total := from.distance_to(to)
		var at := 0.0

		while at < total:
			dashed.push_back(from.lerp(to, at / total))
			dashed.push_back(from.lerp(to, minf(at + length, total) / total))
			at += length * 2.0

	return dashed

func _pixel(x: int, y: int) -> String:
	return "pixel: (%d, %d)" % [x, y]

func _scene() -> SceneRoot:
	var found := scene_view.find_children("*", "SceneRoot", true, false)
	return null if found.is_empty() else found[0]
