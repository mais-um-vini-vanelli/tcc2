extends SubViewportContainer

const LOOK_SENSITIVITY := 0.003 # radians per pixel
const MOVE_SPEED := 5.0 # units per second
const PAN_SPEED := 0.01 # units per pixel

const HANDLE_HALF := 0.15

signal tcc2_node_selected(node: Tcc2Node)
signal gizmo_moved

@onready var camera_3d: Camera3D = $SceneView/Camera3D
@onready var scene_view: SubViewport = $SceneView

var _gizmos: Array[TransformGizmo] = []
var _dragging: TransformGizmo

var _flying := false
var _panning := false
var _yaw := 0.0
var _pitch := 0.0

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_yaw = camera_3d.rotation.y
	_pitch = camera_3d.rotation.x

	for transform in [TranslateGizmo.new(), ScaleGizmo.new(), RotateGizmo.new()]:
		transform.camera = camera_3d
		transform.moved.connect(gizmo_moved.emit)
		scene_view.add_child(transform)
		_gizmos.append(transform)

func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		_on_input_event_mouse_button(event)
	elif event is InputEventMouseMotion:
		_on_input_event_mouse_motion(event)

func _on_input_event_mouse_button(event: InputEventMouseButton) -> void:
	# Otherwise W/A/S/D would also type into a focused SpinBox.
	if event.pressed and _over_viewport():
		var focused := get_viewport().gui_get_focus_owner()
		if focused:
			focused.release_focus()
			
	if event.button_index == MOUSE_BUTTON_RIGHT:
		if event.pressed and _over_viewport():
			_flying = true
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
		elif not event.pressed and _flying:
			_flying = false
			Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	elif event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed and _over_viewport():
			var mouse_position := get_local_mouse_position()

			if Input.is_key_pressed(KEY_SPACE):
				_panning = true
			else:
				for giz in _gizmos:
					var axis := giz.handle_at(mouse_position)

					if axis >= 0:
						_dragging = giz
						giz.begin(axis, camera_3d.project_ray_origin(mouse_position), camera_3d.project_ray_normal(mouse_position))
						break

				if _dragging == null:
					tcc2_node_selected.emit(select_tcc2_node(mouse_position))

			get_viewport().set_input_as_handled()
		elif not event.pressed:
			_panning = false

			if _dragging != null:
				_dragging.end()
				_dragging = null

func set_target(node: Tcc2Node) -> void:
	for one in _gizmos:
		one.target = node

func select_tcc2_node(screen_position: Vector2) -> Tcc2Node:
	var origin := camera_3d.project_ray_origin(screen_position)
	var dir := camera_3d.project_ray_normal(screen_position)
	
	var closest: Tcc2Node = null
	var closest_distance := INF

	for node in scene_view.find_children("*", "Tcc2Node", true, false):
		var distance := _ray_distance(origin, dir, node)
		
		if distance < closest_distance:
			closest_distance = distance
			closest = node

	return closest

func _ray_distance(origin: Vector3, dir: Vector3, node: Tcc2Node) -> float:
	var nearest := INF

	for child in node.get_children(true):
		var geometry_instance := child as GeometryInstance3D
		
		if geometry_instance == null or not geometry_instance.visible:
			continue
		
		var to_local := geometry_instance.global_transform.affine_inverse()
		var local_origin = to_local * origin;
		var local_dir = to_local.basis * dir;
		
		var mesh_instance := child as MeshInstance3D
		
		if mesh_instance != null and mesh_instance.mesh != null:
			if mesh_instance.get_aabb().intersects_ray(to_local * origin, to_local.basis * dir) != null:
				var vertices := mesh_instance.mesh.get_faces()
				
				for i in range(0, vertices.size(), 3):
					var hit = Geometry3D.ray_intersects_triangle(local_origin, local_dir, vertices[i], vertices[i + 1], vertices[i + 2])
					if hit != null:
						nearest = minf(nearest, origin.distance_to(mesh_instance.global_transform * (hit as Vector3)))
		else:
			# Camera and light don't use MeshInstance3D previews
			var aabb := AABB(Vector3.ONE * -HANDLE_HALF, Vector3.ONE * HANDLE_HALF * 2.0)
			
			if aabb.intersects_ray(local_origin, local_dir) != null:
				nearest = minf(nearest, origin.distance_to(geometry_instance.global_position))

	return nearest

func _on_input_event_mouse_motion(event: InputEventMouseMotion) -> void:
	if _dragging != null:
		var mouse := get_local_mouse_position()
		_dragging.drag(camera_3d.project_ray_origin(mouse), camera_3d.project_ray_normal(mouse))
	elif _flying:
		_yaw -= event.relative.x * LOOK_SENSITIVITY
		_pitch = clamp(_pitch - event.relative.y * LOOK_SENSITIVITY, -1.55, 1.55)
		camera_3d.rotation = Vector3(_pitch, _yaw, 0.0)
	elif _panning:
		# Drag the world: the camera moves opposite to the mouse.
		camera_3d.global_position -= camera_3d.global_basis.x * event.relative.x * PAN_SPEED
		camera_3d.global_position += camera_3d.global_basis.y * event.relative.y * PAN_SPEED
	else:
		_hover(get_local_mouse_position())
		
func _hover(mouse: Vector2) -> void:
	var taken := false

	for one in _gizmos:
		var axis := -1 if taken else one.handle_at(mouse)
		taken = taken or axis >= 0
		one.highlight(axis)

func _over_viewport() -> bool:
	if _flying or _panning:
		return true

	return get_global_rect().has_point(get_global_mouse_position())
		
func _process(delta: float) -> void:
	if not _flying:
		return
		
	var dir := Vector3.ZERO
	if Input.is_key_pressed(KEY_W): dir -= camera_3d.global_basis.z # -Z is forward
	if Input.is_key_pressed(KEY_S): dir += camera_3d.global_basis.z
	if Input.is_key_pressed(KEY_A): dir -= camera_3d.global_basis.x
	if Input.is_key_pressed(KEY_D): dir += camera_3d.global_basis.x
	if Input.is_key_pressed(KEY_E): dir += camera_3d.global_basis.y
	if Input.is_key_pressed(KEY_Q): dir -= camera_3d.global_basis.y
	
	if dir != Vector3.ZERO:
		camera_3d.global_position += dir.normalized() * MOVE_SPEED * delta

func _notification(what: int) -> void:
	# Alt-tabbing out while captured would leave the cursor stuck hidden.
	if what == NOTIFICATION_APPLICATION_FOCUS_OUT and _flying:
		_flying = false
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
