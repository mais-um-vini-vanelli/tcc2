@tool
class_name Section extends PanelContainer

var title: String:
	get:
		var header := _header()
		return header.text if header != null else ""
	set(value):
		var header := _header()

		if header != null:
			header.text = value.to_upper()

func _ready() -> void:
	if Engine.is_editor_hint():
		theme = Palette.preview()

	var header := _header()
	var body := _body()

	if header == null or body == null:
		return

	theme_type_variation = &"Subsection" if _nested() else &"Section"

	header.theme_type_variation = &"SectionHeader"
	header.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	header.text = header.text.to_upper()

	if Engine.is_editor_hint():
		return

	var arrow := func(open: bool) -> Texture2D:
		return get_theme_icon(&"fold" if open else &"unfold", &"SectionHeader")

	body.visible = false
	header.button_pressed = false
	header.icon = arrow.call(false)

	header.toggled.connect(func(on: bool):
		body.visible = on
		header.icon = arrow.call(on))

func _nested() -> bool:
	var parent := get_parent()

	while parent != null:
		if parent is Section:
			return true

		parent = parent.get_parent()

	return false

func _header() -> Button:
	return get_node_or_null("Stack/Header") as Button

func _body() -> Control:
	return get_node_or_null("Stack/Body") as Control
