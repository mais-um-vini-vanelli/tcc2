class_name Tcc2TextureLocation extends RefCounted

signal changed

var source: String = "":
	set(value):
		source = value
		changed.emit()

var image: int = 0:
	set(value):
		image = value
		changed.emit()
