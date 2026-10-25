class_name SceneChannel extends RefCounted

signal changed

enum Kind {CONSTANT, MAP}
enum TextureKind {RGB, SCALAR}

class Rgb:
	signal changed

	var constant: Color = Color.BLACK:
		set(value):
			constant = value
			changed.emit()

class Scalar:
	signal changed

	var constant: float = 0.0:
		set(value):
			constant = value
			changed.emit()

var kind: Kind = Kind.CONSTANT:
	set(value):
		kind = value
		changed.emit()

var texture_kind: TextureKind = TextureKind.RGB

var map: int = -1:
	set(value):
		map = value
		changed.emit()

var rgb := Rgb.new()
var scalar := Scalar.new()

func _init(of_type: TextureKind = TextureKind.RGB) -> void:
	texture_kind = of_type
	rgb.changed.connect(changed.emit)
	scalar.changed.connect(changed.emit)

func mapped() -> bool:
	return kind == Kind.MAP and map >= 0
