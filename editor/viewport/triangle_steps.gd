class_name TriangleSteps extends Node3D

signal reported(lines: Array[String])

const STAGE_COLORS := {
	"frustum": Color(1.0, 0.85, 0.2),
	"local": Color(1.0, 0.40, 0.4),
	"clipped": Color(0.3, 1.00, 0.4),
	"screen_plane": Color(0.3, 0.80, 1.0),
	"projection": Color(0.45, 0.20, 0.6),
	"screen": Color(0.8, 0.40, 1.0),
	"world": Color(1.0, 1.00, 1.0),
}

const SCREEN_DEPTH := 1.0

var scene_view: SubViewport

var _mesh := ImmediateMesh.new()
var _instance: MeshInstance3D
var _active := false
var _hidden: Array[MeshInstance3D] = []

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
		_restore()

		var none: Array[String] = []
		reported.emit(none)

func show_triangle(instance: int, face: int) -> void:
	_mesh.clear_surfaces()
	_restore()

	var scene := _scene()

	if scene == null:
		return

	var waiting: Array[String] = _picked(instance, face)
	waiting.append("processando...")
	reported.emit(waiting)

	# Writing the scene blocks everything: let the toast land first.
	await get_tree().process_frame
	await get_tree().process_frame

	if not _active:
		return

	var lines: Array[String] = []
	var bytes := Write.to_tcc2(scene, lines)
	var stages: Dictionary = Tcc2Raster.triangle_stages(bytes, instance, face,
		SCREEN_DEPTH, lines)

	if stages.is_empty():
		reported.emit(lines)
		return

	_isolate(scene)

	_instance.global_transform = scene.global_transform

	_draw(stages)

	var summary: Array[String] = _picked(instance, face)
	summary.append_array(lines)
	reported.emit(summary)

func _draw(stages: Dictionary) -> void:
	_surface("frustum", Mesh.PRIMITIVE_LINES, stages["frustum"])
	_surface("screen_plane", Mesh.PRIMITIVE_LINES, GizmoMesh.loop(stages["screen_plane"]))
	_surface("projection", Mesh.PRIMITIVE_LINES, stages["projection"])

	_surface("local", Mesh.PRIMITIVE_LINES, _dashed(GizmoMesh.loop(stages["local"]), 0.0))
	_surface("world", Mesh.PRIMITIVE_LINES, _dashed(GizmoMesh.loop(stages["world"]), 0.5))

	_surface("clipped", Mesh.PRIMITIVE_TRIANGLES, stages["clipped"])
	_surface("screen", Mesh.PRIMITIVE_TRIANGLES, stages["screen"])

func _surface(stage: String, primitive: int, points: PackedVector3Array) -> void:
	if points.is_empty():
		return

	const FILL_ALPHA := 0.25
	var fill := primitive == Mesh.PRIMITIVE_TRIANGLES

	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_color = Color(STAGE_COLORS[stage], FILL_ALPHA if fill else 1.0)
	material.cull_mode = BaseMaterial3D.CULL_DISABLED

	if fill:
		material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA

	GizmoMesh.surface(_mesh, primitive, material, points)

static func _dashed(lines: PackedVector3Array,
		phase: float) -> PackedVector3Array:
	const DASHES := 24
	const MIN_DASHES := 3

	var dashes := PackedVector3Array()
	var longest := 0.0

	for i in range(0, lines.size() - 1, 2):
		longest = maxf(longest, lines[i].distance_to(lines[i + 1]))

	if longest <= 0.0:
		return dashes

	var step := longest / DASHES

	for i in range(0, lines.size() - 1, 2):
		var a := lines[i]
		var b := lines[i + 1]
		var length := a.distance_to(b)

		# An edge far shorter than the longest one still has to read as dashed.
		var edge_step := minf(step, length / MIN_DASHES)
		var at := phase * edge_step

		while at < length:
			dashes.append_array([a.lerp(b, at / length),
				a.lerp(b, minf(at + edge_step * 0.5, length) / length)])
			at += edge_step

	return dashes

func _isolate(scene: SceneRoot) -> void:
	for node in scene.find_children("*", "SceneGeometry", true, false):
		var preview: MeshInstance3D = (node as SceneGeometry).preview()

		if preview == null or not preview.visible:
			continue

		_hidden.append(preview)
		preview.visible = false

func _restore() -> void:
	for preview in _hidden:
		if is_instance_valid(preview):
			preview.visible = true

	_hidden.clear()

func _picked(instance: int, face: int) -> Array[String]:
	var lines: Array[String] = ["instance: %d" % instance, "face: %d" % face]
	return lines

func _scene() -> SceneRoot:
	var found := scene_view.find_children("*", "SceneRoot", true, false)
	return null if found.is_empty() else found[0]
