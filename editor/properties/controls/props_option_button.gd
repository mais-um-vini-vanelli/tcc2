class_name PropsOptionButton extends OptionButton

func _ready() -> void:
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	alignment = HORIZONTAL_ALIGNMENT_RIGHT
	fit_to_longest_item = false
