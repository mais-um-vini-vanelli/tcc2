extends SubViewportContainer

const LOOK_SENSITIVITY := 0.003 # radians per pixel
const MOVE_SPEED := 5.0 # units per second
const PAN_SPEED := 0.01 # units per pixel
const WRAP_INSET := 2.0 # pixels

const HANDLE_HALF := 0.15


signal tcc2_node_selected(node: Tcc2Node)
signal gizmo_moved
signal triangle_picked(node: Tcc2Geometry, instance: int, face: int)
signal pixel_picked(x: int, y: int)

@onready var camera_3d: Camera3D = $SceneView/Camera3D
@onready var scene_view: SubViewport = $SceneView
@onready var outline: SelectionOutline = $"../SelectionOutline"

var _gizmos: Array[TransformGizmo] = []
var _picker: TrianglePicker
var _dragging: TransformGizmo
var _target: Tcc2Node
var _pov: CameraPov
var _pixels: PixelPicker

var overlay: Control

var _flying := false
var _pan_mode := false
var _pan_drag := false
var _using_pan_cursor := false
var _pan_mouse := Vector2.ZERO
var _yaw := 0.0
var _pitch := 0.0

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_yaw = camera_3d.rotation.y
	_pitch = camera_3d.rotation.x

	camera_3d.cull_mask &= ~Tcc2Node.SELECTION_LAYER

	for transform in [TranslateGizmo.new(), ScaleGizmo.new(), RotateGizmo.new()]:
		transform.camera = camera_3d
		transform.moved.connect(gizmo_moved.emit)
		scene_view.add_child(transform)
		_gizmos.append(transform)

	_picker = TrianglePicker.new()
	_picker.camera = camera_3d
	_picker.scene_view = scene_view
	_picker.picked.connect(triangle_picked.emit)
	scene_view.add_child(_picker)

	_pov = CameraPov.new()
	_pov.scene_view = scene_view
	_pov.container = self
	_pov.editor_camera = camera_3d
	add_child(_pov)

	_pixels = PixelPicker.new()
	_pixels.container = self
	_pixels.scene_view = scene_view
	_pixels.picked.connect(pixel_picked.emit)
	add_child(_pixels)

func _input(event: InputEvent) -> void:
	if _pixel_input(event):
		return

	if event is InputEventMouseButton:
		_on_input_event_mouse_button(event)
	elif event is InputEventMouseMotion:
		_on_input_event_mouse_motion(event)
	elif event is InputEventKey:
		_on_input_event_key(event)

func _on_input_event_key(event: InputEventKey) -> void:
	if event.keycode != KEY_SPACE or not event.pressed or event.echo:
		return

	if _pan_mode:
		_pan_mode = false
		_pan_drag = false
	elif (_over_viewport() and not _flying and _dragging == null
		and not _in_pov() and not _picker.is_active()):
		_pan_mode = true
	else:
		return

	get_viewport().set_input_as_handled()

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
		if event.pressed and _pan_mode and _over_viewport():
			_pan_drag = true
			_pan_mouse = get_viewport().get_mouse_position()
			get_viewport().set_input_as_handled()
		elif event.pressed and _over_viewport():
			var mouse_position := get_local_mouse_position()

			if _picker.click(mouse_position):
				get_viewport().set_input_as_handled()
				return

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
			_pan_drag = false

			if _dragging != null:
				_dragging.end()
				_dragging = null

func _cancel_mouse() -> void:
	if _flying:
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE

	_flying = false
	_pan_mode = false
	_pan_drag = false

	if _picker != null:
		_picker.leave()

	if _pixels != null:
		_pixels.leave()

	if _dragging != null:
		_dragging.end()
		_dragging = null

func _pixel_input(event: InputEvent) -> bool:
	if not _in_pov() and not _pixels.is_active():
		return false

	if event is InputEventMouseMotion:
		if _over_viewport():
			_pixels.hover(get_local_mouse_position())
		else:
			_pixels.leave()
	elif event is InputEventMouseButton:
		var button := event as InputEventMouseButton

		if _over_viewport() and button.button_index == MOUSE_BUTTON_LEFT and button.pressed:
			_pixels.click(get_local_mouse_position())
			get_viewport().set_input_as_handled()
	
	return true

func set_pick_triangle(enabled: bool) -> void:
	_cancel_mouse()
	_picker.set_active(enabled)

func set_pick_pixel(enabled: bool) -> void:
	_cancel_mouse()
	_pixels.set_active(enabled)

func set_target(node: Tcc2Node) -> void:
	if node is Tcc2Scene:
		node = null

	_target = node

	for one in _gizmos:
		one.target = _target

func render_camera() -> Tcc2Camera:
	return _pov.render_camera()

func set_pov(enabled: bool) -> void:
	_cancel_mouse()
	_pov.set_active(enabled)
	outline.set_enabled(not enabled)

	set_target(_target)

func _in_pov() -> bool:
	return _pov.is_active()

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
	elif _pan_drag:
		# Drag the world: the camera moves opposite to the mouse.
		var motion := _pan_motion()
		camera_3d.global_position -= camera_3d.global_basis.x * motion.x * PAN_SPEED
		camera_3d.global_position += camera_3d.global_basis.y * motion.y * PAN_SPEED
	elif _picker.is_active():
		if _over_viewport():
			_picker.hover(get_local_mouse_position())
		else:
			_picker.leave()
	else:
		_hover(get_local_mouse_position())
		
func _pan_motion() -> Vector2:
	var mouse := get_viewport().get_mouse_position()
	var motion := mouse - _pan_mouse
	var wrapped := _wrapped_into_viewport(mouse)

	_pan_mouse = wrapped

	if wrapped != mouse:
		Input.warp_mouse(wrapped)

	return motion

func _wrapped_into_viewport(mouse: Vector2) -> Vector2:
	var rect := Rect2(get_global_transform_with_canvas().origin, size)
	var wrapped := mouse

	if mouse.x <= rect.position.x + WRAP_INSET:
		wrapped.x = rect.end.x - WRAP_INSET * 2.0
	elif mouse.x >= rect.end.x - WRAP_INSET:
		wrapped.x = rect.position.x + WRAP_INSET * 2.0

	if mouse.y <= rect.position.y + WRAP_INSET:
		wrapped.y = rect.end.y - WRAP_INSET * 2.0
	elif mouse.y >= rect.end.y - WRAP_INSET:
		wrapped.y = rect.position.y + WRAP_INSET * 2.0

	return wrapped

func _update_cursor() -> void:
	var pan := _pan_drag or (_pan_mode and _over_viewport())

	if pan == _using_pan_cursor:
		return

	_using_pan_cursor = pan
	Input.set_default_cursor_shape(
		Input.CURSOR_MOVE if pan else Input.CURSOR_ARROW)

func _hover(mouse: Vector2) -> void:
	var taken := false

	for one in _gizmos:
		var axis := -1 if taken else one.handle_at(mouse)
		taken = taken or axis >= 0
		one.highlight(axis)

func _over_viewport() -> bool:
	if _flying or _pan_drag:
		return true

	var mouse := get_global_mouse_position()

	if overlay != null and overlay.get_global_rect().has_point(mouse):
		return false

	return get_global_rect().has_point(mouse)
		
func _process(delta: float) -> void:
	_update_cursor()

	if _in_pov():
		return

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
	# Alt-tabbing out while captured would leave the cursor stuck hidden, and
	# the release that ends a drag or a pan goes to whatever took the focus.
	if what == NOTIFICATION_APPLICATION_FOCUS_OUT:
		_cancel_mouse()
