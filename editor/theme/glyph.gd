class_name Glyph

static func of(size: Vector2i, stamps: Array) -> ImageTexture:
	var canvas := Image.create_empty(size.x, size.y, false, Image.FORMAT_RGBA8)

	for stamp in stamps:
		_stamp(canvas, stamp[0], stamp[1], stamp[2])

	return ImageTexture.create_from_image(canvas)

static func rimmed(texture: Texture2D, color: Color) -> ImageTexture:
	var shape := texture.get_image()
	shape.convert(Image.FORMAT_RGBA8)

	var size := shape.get_size()
	var rim := Image.create_empty(size.x, size.y, false, Image.FORMAT_RGBA8)

	for y in size.y:
		for x in size.x:
			if shape.get_pixel(x, y).a == 0.0 and _touches(shape, x, y):
				rim.set_pixel(x, y, color)

	rim.blend_rect(shape, Rect2i(Vector2i.ZERO, size), Vector2i.ZERO)

	return ImageTexture.create_from_image(rim)

static func _stamp(canvas: Image, texture: Texture2D, color: Color, at: int) -> void:
	var source := texture.get_image()
	source.convert(Image.FORMAT_RGBA8)

	for y in mini(source.get_height(), canvas.get_height()):
		for x in source.get_width():
			var alpha := source.get_pixel(x, y).a

			if alpha > 0.0:
				canvas.set_pixel(at + x, y, Color(color, alpha))

static func _touches(shape: Image, x: int, y: int) -> bool:
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			var at := Vector2i(x + dx, y + dy)

			if at.x < 0 or at.y < 0 or at.x >= shape.get_width() or at.y >= shape.get_height():
				continue

			if shape.get_pixel(at.x, at.y).a > 0.0:
				return true

	return false
