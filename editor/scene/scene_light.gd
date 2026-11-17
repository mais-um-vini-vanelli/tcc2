class_name SceneLight extends SceneObject

enum Kind {POINT, DIRECTIONAL, SPOT, ENVIRONMENT}

class Point:
	var light: SceneLight

	var color: Color = Color.WHITE:
		set(value):
			color = value
			light._rebuild_preview()

	var intensity: float = 1.0:
		set(value):
			intensity = value
			light._apply_intensity()

class Directional:
	var light: SceneLight

	var color: Color = Color.WHITE:
		set(value):
			color = value
			light._rebuild_preview()

	var intensity: float = 1.0:
		set(value):
			intensity = value
			light._apply_intensity()

class Spot:
	var light: SceneLight

	var inner_angle: float = 20.0
	var outer_angle: float = 30.0:
		set(value):
			outer_angle = value
			light._rebuild_preview()

	var color: Color = Color.WHITE:
		set(value):
			color = value
			light._rebuild_preview()

	var intensity: float = 1.0:
		set(value):
			intensity = value
			light._apply_intensity()

class Environment_:
	var light: SceneLight

	var color: Color = Color.WHITE:
		set(value):
			color = value
			light._rebuild_preview()

	var intensity: float = 1.0:
		set(value):
			intensity = value
			light._apply_intensity()

var kind: Kind = Kind.POINT:
	set(value):
		kind = value
		_rebuild_preview()

var point := Point.new()
var directional := Directional.new()
var spot := Spot.new()
var environment := Environment_.new()

var _icon: Sprite3D
var _gizmo: MeshInstance3D
var _preview: Light3D
var _shadows := true

func _init() -> void:
	point.light = self
	directional.light = self
	spot.light = self
	environment.light = self

func _ready() -> void:
	super()

	_icon = _add_icon(preload("res://icons/light.svg"))

	_gizmo = MeshInstance3D.new()
	_gizmo.layers = ICONS_LAYER
	_gizmo.material_override = _selected_gizmo_material()
	_gizmo.visible = false
	add_child(_gizmo, false, Node.INTERNAL_MODE_FRONT)

	_rebuild_preview()

func model():
	match kind:
		Kind.DIRECTIONAL:
			return directional

		Kind.SPOT:
			return spot

		Kind.ENVIRONMENT:
			return environment

	return point

func sync_preview(scene: SceneRoot) -> void:
	_shadows = scene == null or scene.shadows

	if _preview != null:
		_preview.shadow_enabled = _shadows

func set_selected(value: bool) -> void:
	if _icon != null:
		_set_icon_selected(_icon, value, ICON_COLOR)
		_gizmo.visible = value

func _rebuild_preview() -> void:
	if _icon == null:
		return

	var mod = model()

	_gizmo.mesh = _gizmo_mesh()

	if _preview != null:
		remove_child(_preview)
		_preview.queue_free()
		_preview = null

	_preview = _light_preview()

	if _preview == null:
		return

	_preview.light_color = mod.color
	_preview.light_energy = mod.intensity
	_preview.shadow_enabled = _shadows
	_preview.light_cull_mask = 1

	add_child(_preview, false, Node.INTERNAL_MODE_FRONT)

func _apply_intensity() -> void:
	if _preview != null:
		_preview.light_energy = model().intensity

func _light_preview() -> Light3D:
	match kind:
		Kind.POINT:
			return OmniLight3D.new()

		Kind.DIRECTIONAL:
			return DirectionalLight3D.new()

		Kind.SPOT:
			var preview := SpotLight3D.new()
			preview.spot_angle = spot.outer_angle
			return preview

	return null

func _gizmo_mesh() -> Mesh:
	match kind:
		Kind.DIRECTIONAL:
			return _arrow_mesh()

		Kind.SPOT:
			return _cone_mesh()

	return null

func _arrow_mesh() -> Mesh:
	const ARROW_LENGTH := 1.2
	const ARROW_HEAD_LENGTH := 0.45
	const ARROW_HEAD_RADIUS := 0.28
	const ARROW_SHAFT_RADIUS := 0.10
	const ARROW_SEGMENTS := 4
	const ARROW_RAYS := 4

	var tip := Vector3.FORWARD * ARROW_LENGTH
	var neck := - (ARROW_LENGTH - ARROW_HEAD_LENGTH)

	var head_rim := GizmoMesh.ring(ARROW_HEAD_RADIUS, neck, ARROW_SEGMENTS)
	var shaft_far := GizmoMesh.ring(ARROW_SHAFT_RADIUS, neck, ARROW_SEGMENTS)
	var shaft_near := GizmoMesh.ring(ARROW_SHAFT_RADIUS, 0.0, ARROW_SEGMENTS)

	var lines := GizmoMesh.loop(head_rim)
	lines.append_array(GizmoMesh.loop(shaft_near))

	@warning_ignore("integer_division")
	var step := ARROW_SEGMENTS / ARROW_RAYS

	for i in ARROW_RAYS:
		var k := i * step
		lines.append_array([head_rim[k], tip, head_rim[k], shaft_far[k],
			shaft_far[k], shaft_near[k]])

	var mesh := ImmediateMesh.new()
	GizmoMesh.surface(mesh, Mesh.PRIMITIVE_LINES, _gizmo_material(), lines)

	return mesh

func _cone_mesh() -> Mesh:
	const CONE_LENGTH := 1.2
	const CONE_SEGMENTS := 8
	const CONE_RAYS := 8

	var radius := CONE_LENGTH * tan(deg_to_rad(minf(spot.outer_angle, 89.0)))
	var rim := GizmoMesh.ring(radius, -CONE_LENGTH, CONE_SEGMENTS)

	var lines := GizmoMesh.loop(rim)

	for i in CONE_RAYS:
		@warning_ignore("integer_division")
		lines.append_array([Vector3.ZERO, rim[i * CONE_SEGMENTS / CONE_RAYS]])

	var mesh := ImmediateMesh.new()
	GizmoMesh.surface(mesh, Mesh.PRIMITIVE_LINES, _gizmo_material(), lines)

	return mesh
