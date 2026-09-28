class_name Tcc2Texture extends Resource

@export_file("*.glb", "*.png", "*.jpg", "*.jpeg") var source: String = "":
	set(value):
		source = value
		notify_property_list_changed()
		emit_changed()

@export var image: int = 0:
	set(value):
		image = value
		emit_changed()

# Shared by every Tcc2Texture: an image file is read once.
static var _cache := {}

func resolve() -> Texture2D:
	if source.is_empty():
		return null

	var images: Array = (Tcc2Glb.images(source) if source.get_extension().to_lower() == "glb"
		else _read(source))

	return images[image] if image >= 0 and image < images.size() else null

# An image file holds exactly one image, so there is nothing to index into.
func _validate_property(property: Dictionary) -> void:
	if property.name == "image" and source.get_extension().to_lower() != "glb":
		property.usage &= ~PROPERTY_USAGE_EDITOR

static func _read(path: String) -> Array:
	if not _cache.has(path):
		var picture := Image.load_from_file(path)

		if picture == null:
			push_error("could not read %s" % path)
			_cache[path] = []
		else:
			_cache[path] = [ImageTexture.create_from_image(picture)]

	return _cache[path]
