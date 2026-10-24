class_name ViewportPane extends SubViewportContainer

const HANDLE_HALF := 0.15

enum Mode {EDIT, POV, PICK_TRIANGLE, PICK_PIXEL}

signal scene_object_selected(node: SceneObject)
signal gizmo_moved
signal triangle_picked(node: SceneGeometry, instance: int, face: int)
signal pixel_picked(x: int, y: int)

@onready var camera_3d: Camera3D = $SceneView/Camera3D
@onready var scene_view: SubViewport = $SceneView
@onready var outline: SelectionOutline = $"../SelectionOutline"

var _gizmo: TransformGizmo
var _picker: TrianglePicker
var _target: SceneObject
var _pov: CameraPov
var _pixels: PixelPicker
var _nav: CameraNav
var _mode := Mode.EDIT

var overlay: Control

var _cursor := Input.CURSOR_ARROW

func _ready() -> void:
	camera_3d.cull_mask &= ~SceneObject.SELECTION_LAYER

	_gizmo = TransformGizmo.new()
	_gizmo.camera = camera_3d
	_gizmo.moved.connect(gizmo_moved.emit)
	scene_view.add_child(_gizmo)

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

	_nav = CameraNav.new()
	_nav.camera = camera_3d
	_nav.container = self
	add_child(_nav)

func _input(event: InputEvent) -> void:
	var over := _over_viewport()
	var button := event as InputEventMouseButton

	if button != null and button.pressed and over:
		var focused := get_viewport().gui_get_focus_owner()

		if focused:
			focused.release_focus()

	match _mode:
		Mode.PICK_PIXEL:
			_pixel_input(event, over)
		Mode.PICK_TRIANGLE:
			_triangle_input(event, over)
		Mode.POV:
			pass # The render camera owns the pane: the mouse does nothing.
		Mode.EDIT:
			_edit_input(event, over)

func _pixel_input(event: InputEvent, over: bool) -> void:
	if event is InputEventMouseMotion:
		if over:
			_pixels.hover(get_local_mouse_position())
		else:
			_pixels.leave()
	elif event is InputEventMouseButton:
		var button := event as InputEventMouseButton

		if over and button.button_index == MOUSE_BUTTON_LEFT and button.pressed:
			_pixels.click(get_local_mouse_position())
			get_viewport().set_input_as_handled()

func _triangle_input(event: InputEvent, over: bool) -> void:
	if event is InputEventMouseMotion:
		if over:
			_picker.hover(get_local_mouse_position())
		else:
			_picker.leave()
	elif event is InputEventMouseButton:
		var button := event as InputEventMouseButton

		if over and button.button_index == MOUSE_BUTTON_LEFT and button.pressed:
			_picker.click(get_local_mouse_position())
			get_viewport().set_input_as_handled()

func _edit_input(event: InputEvent, over: bool) -> void:
	if _nav_input(event, over):
		return

	if event is InputEventMouseButton:
		_mouse_button(event as InputEventMouseButton, over)
	elif event is InputEventMouseMotion:
		_mouse_motion()

func _nav_input(event: InputEvent, over: bool) -> bool:
	if not _nav.is_active():
		if _gizmo.is_dragging() or (not over and not _nav.is_pan_armed()):
			return false

	if not _nav.handle(event, over):
		return false

	if event is not InputEventMouseMotion:
		get_viewport().set_input_as_handled()

	return true

func _mouse_button(event: InputEventMouseButton, over: bool) -> void:
	if event.button_index != MOUSE_BUTTON_LEFT:
		return

	if not event.pressed:
		_gizmo.release()
		return

	if not over or _nav.is_active() or _gizmo.is_dragging():
		return

	var mouse_position := get_local_mouse_position()

	var grabbed := _gizmo.grab(mouse_position,
		camera_3d.project_ray_origin(mouse_position),
		camera_3d.project_ray_normal(mouse_position))

	if not grabbed:
		scene_object_selected.emit(select_scene_object(mouse_position))

	get_viewport().set_input_as_handled()

func _mouse_motion() -> void:
	var mouse := get_local_mouse_position()

	if _gizmo.is_dragging():
		_gizmo.drag(camera_3d.project_ray_origin(mouse),
			camera_3d.project_ray_normal(mouse))
	else:
		_gizmo.hover(mouse)

func _cancel_mouse() -> void:
	if _nav == null:
		return

	_nav.cancel()
	_picker.leave()
	_pixels.leave()
	_gizmo.release()

	_update_pointer()

func set_mode(mode: Mode) -> void:
	if _mode == mode:
		return

	_mode = mode
	_cancel_mouse()

	var pov := mode == Mode.POV or mode == Mode.PICK_PIXEL

	_pov.set_active(pov)
	_picker.set_active(mode == Mode.PICK_TRIANGLE)
	_pixels.set_active(mode == Mode.PICK_PIXEL)
	outline.set_enabled(not pov)

	set_target(_target)

func refresh_pov() -> void:
	if _mode == Mode.POV or _mode == Mode.PICK_PIXEL:
		_pov.set_active(true)

func set_target(node: SceneObject) -> void:
	if node is SceneRoot:
		node = null

	_target = node
	_gizmo.target = _target

func render_camera() -> SceneCamera:
	return _pov.render_camera()

func select_scene_object(screen_position: Vector2) -> SceneObject:
	var origin := camera_3d.project_ray_origin(screen_position)
	var dir := camera_3d.project_ray_normal(screen_position)

	var closest: SceneObject = null
	var closest_distance := INF

	for node in scene_view.find_children("*", "SceneObject", true, false):
		var distance := _ray_distance(origin, dir, node)

		if distance < closest_distance:
			closest_distance = distance
			closest = node

	return closest

func _ray_distance(origin: Vector3, dir: Vector3, node: SceneObject) -> float:
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

func _update_pointer() -> void:
	if _nav == null:
		return

	var shape := _pixels.cursor()

	if shape == Input.CURSOR_ARROW:
		shape = _picker.cursor()

	if shape == Input.CURSOR_ARROW:
		shape = _nav.cursor(_over_viewport())

	if shape != _cursor:
		_cursor = shape
		Input.set_default_cursor_shape(shape)

	var mouse := _nav.mouse_mode()

	if mouse == Input.MOUSE_MODE_VISIBLE:
		mouse = _pixels.mouse_mode()

	if mouse != Input.mouse_mode:
		Input.mouse_mode = mouse

func _over_viewport() -> bool:
	var mouse := get_global_mouse_position()

	if _over_overlay(mouse):
		return false

	return get_global_rect().has_point(mouse)

func _over_overlay(mouse: Vector2) -> bool:
	if overlay == null:
		return false

	for child in overlay.get_children():
		var widget := child as Control

		if widget == null or not widget.visible:
			continue

		if widget.get_global_rect().has_point(mouse):
			return true

	return false

func _process(_delta: float) -> void:
	_update_pointer()

func _notification(what: int) -> void:
	# Alt-tabbing out while captured would leave the cursor stuck hidden, and
	# the release that ends a drag or a pan goes to whatever took the focus.
	if what == NOTIFICATION_APPLICATION_FOCUS_OUT:
		_cancel_mouse()
