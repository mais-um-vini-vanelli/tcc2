class_name Tcc2ScalarValue extends Tcc2Value

@export_range(0.0, 1.0) var constant: float = 0.0:
	set(value):
		constant = value
		emit_changed()

@export_enum("Red", "Green", "Blue") var channel: int = 0:
	set(value):
		channel = value
		emit_changed()

func _validate_property(property: Dictionary) -> void:
	super(property)

	if property.name == "channel" and kind == Kind.CONSTANT:
		property.usage &= ~PROPERTY_USAGE_EDITOR
