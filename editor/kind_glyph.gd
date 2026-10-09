class_name Tcc2KindGlyph

const HEIGHT := 10
const WIDTH := Tcc2Theme.MARKER_WIDTH + Tcc2Theme.GAP + Tcc2Theme.KIND_WIDTH

enum Fold {
	LEAF,
	OPEN,
	SHUT,
}

static var _made := {}

static func of(kind: Texture2D, color: Color, fold: Fold, selected: bool) -> Texture2D:
	var key := "%s|%d|%s" % [kind.resource_path, fold, selected]

	if not _made.has(key):
		_made[key] = _compose(kind, color, fold, selected)

	return _made[key]

static func _compose(kind: Texture2D, color: Color, fold: Fold,
		selected: bool) -> ImageTexture:
	var canvas := Image.create_empty(WIDTH, HEIGHT, false, Image.FORMAT_RGBA8)

	if fold != Fold.LEAF:
		var marker: Texture2D = Tcc2Theme.FOLD if fold == Fold.OPEN else Tcc2Theme.UNFOLD
		_stamp(canvas, marker, 0, Tcc2Theme.INK if selected else Tcc2Theme.FG)

	_stamp(canvas, kind, Tcc2Theme.MARKER_WIDTH + Tcc2Theme.GAP,
		color.darkened(0.55) if selected else color)

	return ImageTexture.create_from_image(canvas)

static func _stamp(canvas: Image, texture: Texture2D, at: int, color: Color) -> void:
	var source := texture.get_image()
	source.convert(Image.FORMAT_RGBA8)

	for y in mini(source.get_height(), HEIGHT):
		for x in source.get_width():
			var alpha := source.get_pixel(x, y).a

			if alpha > 0.0:
				canvas.set_pixel(at + x, y, Color(color, alpha))
