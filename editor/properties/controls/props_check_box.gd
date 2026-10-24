class_name PropsCheckBox extends CheckBox

const ON := "[X]"
const OFF := "[ ]"

var value: bool:
	get:
		return button_pressed
	set(on):
		set_pressed_no_signal(on)
		_refresh()

func _ready() -> void:
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	alignment = HORIZONTAL_ALIGNMENT_CENTER

	toggled.connect(func(_on: bool): _refresh())
	_refresh()

func _refresh() -> void:
	if not is_node_ready():
		return

	_pad()

	text = ON if button_pressed else OFF
	custom_minimum_size.x = _reserve()

func _pad() -> void:
	var base := get_theme_font(&"font")

	if base is FontVariation and base.base_font != null:
		base = base.base_font

	if base == null:
		return

	var size := get_theme_font_size(&"font_size")
	var gap := int(base.get_string_size(ON, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x
		- base.get_string_size(OFF, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x)

	if gap <= 0:
		return

	var padded := FontVariation.new()
	padded.base_font = base
	padded.set_spacing(TextServer.SPACING_SPACE, gap)

	add_theme_font_override(&"font", padded)

func _reserve() -> float:
	var font := get_theme_font(&"font")
	var size := get_theme_font_size(&"font_size")

	if font == null:
		return 0.0

	var widest := maxf(
		font.get_string_size(ON, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x,
		font.get_string_size(OFF, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x)
	
	return ceilf(widest + get_theme_stylebox(&"normal").get_minimum_size().x)
