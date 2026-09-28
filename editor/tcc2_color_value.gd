class_name Tcc2ColorValue extends Tcc2Value

@export var constant: Color = Color.BLACK:
	set(value):
		constant = value
		emit_changed()
