class_name TrianglePicker extends Node3D

signal picked(node: SceneGeometry, instance: int, face: int)

var scene_view: SubViewport
var camera: Camera3D

var _active := false
var _node: SceneGeometry
var _face := -1
var _highlight: MeshInstance3D
var _shape := ImmediateMesh.new()
var _faces := {}
var _pending := Vector2.INF

func _ready() -> void:
	_highlight = MeshInstance3D.new()
	_highlight.mesh = _shape
	_highlight.material_override = _highlight_material()
	_highlight.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_highlight.visible = false
	add_child(_highlight)

func is_active() -> bool:
	return _active

func set_active(value: bool) -> void:
	if _active == value:
		return

	_active = value
	_pending = Vector2.INF
	_forget()

	if not value:
		_faces.clear()
		Input.set_default_cursor_shape(Input.CURSOR_ARROW)

func hover(mouse: Vector2) -> void:
	if _active:
		_pending = mouse

func _process(_delta: float) -> void:
	if not _active or _pending == Vector2.INF:
		return

	var hit := _pick(_pending)
	_pending = Vector2.INF

	if hit.is_empty():
		_forget()
		Input.set_default_cursor_shape(Input.CURSOR_FORBIDDEN)
	else:
		_remember(hit["node"], hit["face"])
		Input.set_default_cursor_shape(Input.CURSOR_CROSS)

## Mouse left the viewport. The cursor shape is global, so it has to go back.
func leave() -> void:
	if not _active:
		return

	_pending = Vector2.INF
	_forget()
	Input.set_default_cursor_shape(Input.CURSOR_ARROW)

## True when the click was the picker's, so the viewport leaves it alone.
func click(mouse: Vector2) -> bool:
	if not _active:
		return false

	var hit := _pick(mouse)

	# A click on nothing stays in picking mode: it just missed every mesh.
	if hit.is_empty():
		return true

	var node: SceneGeometry = hit["node"]
	var face: int = hit["face"]
	var instance := instance_index(_scene(), node)

	set_active(false)
	picked.emit(node, instance, face)

	return true

## The index Rust's `Scene::flatten` gives this node: a pre-order walk of the
## tree, counting every geometry node the exporter writes with a mesh.
static func instance_index(root: Node, target: SceneGeometry) -> int:
	return _instances(root).find(target)

static func _instances(root: Node) -> Array[SceneGeometry]:
	var out: Array[SceneGeometry] = []
	_descend(root, out)
	return out

static func _descend(node: Node, out: Array[SceneGeometry]) -> void:
	for child in node.get_children():
		if child is not SceneObject:
			continue

		var geometry := child as SceneGeometry

		if geometry != null and _exports_mesh(geometry):
			out.append(geometry)

		_descend(child, out)

static func _exports_mesh(node: SceneGeometry) -> bool:
	if node.kind != SceneGeometry.Kind.MESH:
		return true

	if node.mesh.source.is_empty():
		return false

	return Assets.surface(node.mesh.source, node.mesh.mesh,
		node.mesh.surface) != null

## The nearest mesh face along the ray, as {"node": ..., "face": ...}.
func _pick(mouse: Vector2) -> Dictionary:
	if camera == null or scene_view == null:
		return {}

	var origin := camera.project_ray_origin(mouse)
	var dir := camera.project_ray_normal(mouse)

	var best := {}
	var best_distance := INF

	for node in _instances(_scene()):
		var preview := node.preview()

		if preview == null or preview.mesh == null or not preview.visible:
			continue

		var to_local := preview.global_transform.affine_inverse()
		var local_origin := to_local * origin
		var local_dir := to_local.basis * dir

		if preview.get_aabb().intersects_ray(local_origin, local_dir) == null:
			continue

		var faces := _faces_of(preview.mesh)

		for i in range(0, faces.size(), 3):
			var hit = Geometry3D.ray_intersects_triangle(local_origin,
				local_dir, faces[i], faces[i + 1], faces[i + 2])

			if hit == null:
				continue

			var point: Vector3 = preview.global_transform * (hit as Vector3)
			var distance := origin.distance_to(point)

			if distance < best_distance:
				best_distance = distance
				best = {"node": node, "face": i / 3}

	return best

func _remember(node: SceneGeometry, face: int) -> void:
	if _node == node and _face == face:
		return

	_node = node
	_face = face

	var preview := node.preview()
	var faces := _faces_of(preview.mesh)
	var i := face * 3

	_shape.clear_surfaces()
	_shape.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	_shape.surface_add_vertex(faces[i])
	_shape.surface_add_vertex(faces[i + 1])
	_shape.surface_add_vertex(faces[i + 2])
	_shape.surface_end()

	_highlight.global_transform = preview.global_transform
	_highlight.visible = true

func _forget() -> void:
	_node = null
	_face = -1

	if _highlight != null:
		_highlight.visible = false
		_shape.clear_surfaces()

func _faces_of(mesh: Mesh) -> PackedVector3Array:
	if not _faces.has(mesh):
		_faces[mesh] = mesh.get_faces()

	return _faces[mesh]

## The SceneRoot the exporter walks, resolved per call because opening a file
## replaces it.
func _scene() -> Node:
	var found := scene_view.find_children("*", "SceneRoot", true, false)
	return scene_view if found.is_empty() else found[0]

func _highlight_material() -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mat.albedo_color = SceneObject.SELECTION_COLOR
	mat.no_depth_test = true
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.render_priority = 2
	return mat
