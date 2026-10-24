class_name PropsColorPickerButton extends ColorPickerButton

func _ready() -> void:
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var size := get_theme_font_size(&"font_size")

	custom_minimum_size.y = (get_theme_font(&"font").get_height(size)
		+ get_theme_stylebox(&"normal").get_minimum_size().y)
