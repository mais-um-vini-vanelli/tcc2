# One channel of a material: a constant, or a texture to look it up in.
class_name Tcc2Value extends Resource

enum Kind {CONSTANT, TEXTURE}

@export_enum("Constant", "Texture") var kind: int = Kind.CONSTANT:
	set(value):
		kind = value
		notify_property_list_changed()
		emit_changed()

@export var texture: Tcc2Texture = Tcc2Texture.new():
	set(value):
		texture = value if value != null else Tcc2Texture.new()
		_listen()
		emit_changed()

func _init() -> void:
	_listen()

func resolve() -> Texture2D:
	return texture.resolve() if kind == Kind.TEXTURE else null

func _validate_property(property: Dictionary) -> void:
	# Hide properties not relevant to the selected kind.
	if property.name == "constant" and kind == Kind.TEXTURE:
		property.usage &= ~PROPERTY_USAGE_EDITOR
	elif property.name == "texture" and kind == Kind.CONSTANT:
		property.usage &= ~PROPERTY_USAGE_EDITOR

func _listen() -> void:
	if not texture.changed.is_connected(emit_changed):
		texture.changed.connect(emit_changed)
