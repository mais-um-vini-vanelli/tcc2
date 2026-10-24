class_name CameraNav extends Node

const LOOK_SENSITIVITY := 0.003 # radians per pixel
const MOVE_SPEED := 5.0 # units per second
const PAN_SPEED := 0.01 # units per pixel
const WRAP_INSET := 2.0 # pixels

var camera: Camera3D
var container: SubViewportContainer

var _flying := false
var _pan_armed := false
var _pan_drag := false
var _pan_mouse := Vector2.ZERO
var _yaw := 0.0
var _pitch := 0.0

func _ready() -> void:
	_yaw = camera.rotation.y
	_pitch = camera.rotation.x

func is_active() -> bool:
	return _flying or _pan_drag

func is_pan_armed() -> bool:
	return _pan_armed

func cancel() -> void:
	_flying = false
	_pan_armed = false
	_pan_drag = false

func handle(event: InputEvent, over: bool) -> bool:
	if event is InputEventMouseButton:
		return _button(event, over)
	elif event is InputEventMouseMotion:
		return _motion(event)
	elif event is InputEventKey:
		return _key(event, over)

	return false

func cursor(over: bool) -> Input.CursorShape:
	if _pan_drag or (_pan_armed and over):
		return Input.CURSOR_MOVE

	return Input.CURSOR_ARROW

func mouse_mode() -> Input.MouseMode:
	if _flying:
		return Input.MOUSE_MODE_CAPTURED

	return Input.MOUSE_MODE_VISIBLE

func _button(event: InputEventMouseButton, over: bool) -> bool:
	if event.button_index == MOUSE_BUTTON_RIGHT:
		if event.pressed:
			if not over:
				return false

			_flying = true
			return true

		if _flying:
			_flying = false
			return true
	elif event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			if not _pan_armed or not over:
				return false

			_pan_drag = true
			_pan_mouse = get_viewport().get_mouse_position()
			return true

		if _pan_drag:
			_pan_drag = false
			return true

	return false

func _motion(event: InputEventMouseMotion) -> bool:
	if _flying:
		_yaw -= event.relative.x * LOOK_SENSITIVITY
		_pitch = clamp(_pitch - event.relative.y * LOOK_SENSITIVITY, -1.55, 1.55)
		camera.rotation = Vector3(_pitch, _yaw, 0.0)
		return true

	if _pan_drag:
		var motion := _pan_motion()
		camera.global_position -= camera.global_basis.x * motion.x * PAN_SPEED
		camera.global_position += camera.global_basis.y * motion.y * PAN_SPEED
		return true

	return false

func _key(event: InputEventKey, over: bool) -> bool:
	if event.keycode != KEY_SPACE or not event.pressed or event.echo:
		return false

	# Space always turns pan off, wherever the pointer is.
	if _pan_armed:
		_pan_armed = false
		_pan_drag = false
	elif over:
		_pan_armed = true
	else:
		return false

	return true

func _pan_motion() -> Vector2:
	var mouse := get_viewport().get_mouse_position()
	var motion := mouse - _pan_mouse
	var wrapped := _wrapped_into_viewport(mouse)

	_pan_mouse = wrapped

	if wrapped != mouse:
		Input.warp_mouse(wrapped)

	return motion

func _wrapped_into_viewport(mouse: Vector2) -> Vector2:
	var rect := Rect2(container.get_global_transform_with_canvas().origin,
		container.size)
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

func _process(delta: float) -> void:
	if not _flying:
		return

	var dir := Vector3.ZERO
	if Input.is_key_pressed(KEY_W): dir -= camera.global_basis.z # -Z is forward
	if Input.is_key_pressed(KEY_S): dir += camera.global_basis.z
	if Input.is_key_pressed(KEY_A): dir -= camera.global_basis.x
	if Input.is_key_pressed(KEY_D): dir += camera.global_basis.x
	if Input.is_key_pressed(KEY_E): dir += camera.global_basis.y
	if Input.is_key_pressed(KEY_Q): dir -= camera.global_basis.y

	if dir != Vector3.ZERO:
		camera.global_position += dir.normalized() * MOVE_SPEED * delta
