class_name PixelPicker extends Node

signal picked(x: int, y: int)

const SPAN := 11
const ZOOM := 14
const SIDE := SPAN * ZOOM
const LABEL_HEIGHT := 18

var container: SubViewportContainer
var scene_view: SubViewport

var _active := false
var _loupe: Control
var _zoom: TextureRect
var _label: Label

func is_active() -> bool:
	return _active

func set_active(value: bool) -> void:
	if _active == value:
		return

	_active = value

	if value:
		_build()
	else:
		_rest()

	_loupe.visible = false

## Mouse moved inside the viewport, in container-local coordinates.
func hover(mouse: Vector2) -> void:
	if not _active:
		return

	var pixel := _at(mouse)

	if pixel.x < 0:
		# Inside the pane but outside the frame: the letterbox has no pixels.
		_loupe.visible = false
		_rest(Input.CURSOR_FORBIDDEN)
		return

	_place(mouse, pixel)
	_loupe.visible = true
	
	Input.mouse_mode = Input.MOUSE_MODE_HIDDEN

## Mouse left the viewport. The cursor shape is global, so it has to go back.
func leave() -> void:
	if not _active:
		return

	_loupe.visible = false
	_rest()

## True when the click was the picker's, so the viewport leaves it alone.
func click(mouse: Vector2) -> bool:
	if not _active:
		return false

	var pixel := _at(mouse)

	if pixel.x >= 0:
		set_active(false)
		picked.emit(pixel.x, pixel.y)

	return true

## The render pixel under a container-local position, or (-1, -1) outside the
## frame.
func _at(mouse: Vector2) -> Vector2i:
	var size := scene_view.size
	var pixel := Vector2i(floori(mouse.x), floori(mouse.y))

	if pixel.x < 0 or pixel.y < 0 or pixel.x >= size.x or pixel.y >= size.y:
		return Vector2i(-1, -1)

	return pixel

## Centres the loupe on the cursor and scrolls the frame inside it so that the
## picked pixel lands in the middle, under the reticle.
func _place(mouse: Vector2, pixel: Vector2i) -> void:
	# The mouse arrives in render pixels; the loupe lives in the pane, which
	# the preview's scale and letterbox separate from them.
	var at := container.position + mouse * container.scale
	_loupe.position = (at - Vector2(SIDE, SIDE) * 0.5).round()

	# Drawn at one screen pixel per render pixel, then scaled: the texture is
	# the live frame, so its size follows the scene's output.
	_zoom.size = Vector2(scene_view.size)
	_zoom.position = (Vector2(SIDE, SIDE) * 0.5
		- (Vector2(pixel) + Vector2(0.5, 0.5)) * ZOOM)

	_label.text = "%d, %d" % [pixel.x, pixel.y]

func _rest(shape := Input.CURSOR_ARROW) -> void:
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	Input.set_default_cursor_shape(shape)

## Built on first use and kept, hidden, afterwards. It goes in the pane rather
## than the container so that the preview's scale does not magnify it too.
func _build() -> void:
	if _loupe != null:
		return

	_loupe = Control.new()
	_loupe.size = Vector2(SIDE, SIDE + LABEL_HEIGHT)
	_loupe.mouse_filter = Control.MOUSE_FILTER_IGNORE

	var window := Panel.new()
	window.size = Vector2(SIDE, SIDE)
	window.clip_contents = true
	window.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_loupe.add_child(window)

	_zoom = TextureRect.new()
	_zoom.texture = scene_view.get_texture()
	_zoom.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_zoom.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_zoom.scale = Vector2(ZOOM, ZOOM)
	_zoom.mouse_filter = Control.MOUSE_FILTER_IGNORE
	window.add_child(_zoom)

	# Drawn over the frame, around the middle pixel: the one a click takes.
	var reticle := Panel.new()
	reticle.size = Vector2(ZOOM, ZOOM)
	reticle.position = (Vector2(SIDE, SIDE) - reticle.size) * 0.5
	reticle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var outline := StyleBoxFlat.new()
	outline.bg_color = Color.TRANSPARENT
	outline.border_color = SceneObject.SELECTION_COLOR
	outline.set_border_width_all(1)

	reticle.add_theme_stylebox_override("panel", outline)
	window.add_child(reticle)

	_label = Label.new()
	_label.position = Vector2(0, SIDE)
	_label.size = Vector2(SIDE, LABEL_HEIGHT)
	_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_label.add_theme_stylebox_override(&"normal",
		container.get_theme_stylebox(&"panel", &"Panel"))
	_loupe.add_child(_label)

	container.get_parent().add_child(_loupe)
