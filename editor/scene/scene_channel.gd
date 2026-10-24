class_name SceneChannel extends RefCounted

signal changed

enum Kind {CONSTANT, TEXTURE}
enum TextureKind {RGB, SCALAR}

class Rgb:
	signal changed

	var constant: Color = Color.BLACK:
		set(value):
			constant = value
			changed.emit()

	var factor: Color = Color.WHITE:
		set(value):
			factor = value
			changed.emit()

class Scalar:
	signal changed

	var constant: float = 0.0:
		set(value):
			constant = value
			changed.emit()

	var factor: float = 1.0:
		set(value):
			factor = value
			changed.emit()

	var channel: int = 0:
		set(value):
			channel = value
			changed.emit()

var kind: Kind = Kind.CONSTANT:
	set(value):
		kind = value
		changed.emit()

var texture_kind: TextureKind = TextureKind.RGB

var texture: TextureLocation = TextureLocation.new():
	set(value):
		texture = value if value != null else TextureLocation.new()
		_listen()
		changed.emit()

var rgb := Rgb.new()
var scalar := Scalar.new()

func _init(of_type: TextureKind = TextureKind.RGB) -> void:
	texture_kind = of_type
	rgb.changed.connect(changed.emit)
	scalar.changed.connect(changed.emit)
	_listen()

func _listen() -> void:
	if not texture.changed.is_connected(changed.emit):
		texture.changed.connect(changed.emit)
