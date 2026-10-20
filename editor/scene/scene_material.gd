class_name SceneMaterial extends RefCounted

signal changed

var albedo: SceneChannel = SceneChannel.new():
	set(value):
		albedo = value if value != null else SceneChannel.new()
		_listen(albedo)
		changed.emit()

var metallic: SceneChannel = SceneChannel.new(SceneChannel.TextureKind.SCALAR):
	set(value):
		metallic = value if value != null else SceneChannel.new(SceneChannel.TextureKind.SCALAR)
		_listen(metallic)
		changed.emit()

var roughness: SceneChannel = SceneChannel.new(SceneChannel.TextureKind.SCALAR):
	set(value):
		roughness = value if value != null else SceneChannel.new(SceneChannel.TextureKind.SCALAR)
		_listen(roughness)
		changed.emit()

var emissive: SceneChannel = SceneChannel.new():
	set(value):
		emissive = value if value != null else SceneChannel.new()
		_listen(emissive)
		changed.emit()

var normal: TextureLocation = TextureLocation.new():
	set(value):
		normal = value if value != null else TextureLocation.new()
		_listen(normal)
		changed.emit()

# Whitted's model, orthogonal to the above.
var transparency: float = 0.0:
	set(value):
		transparency = value
		changed.emit()

var refractive_index: float = 1.5:
	set(value):
		refractive_index = value
		changed.emit()

# The setters don't run for the declarations' initial values.
func _init() -> void:
	albedo.rgb.constant = Color(0.8, 0.8, 0.8)
	roughness.scalar.constant = 0.5

	for channel in [albedo, metallic, roughness, emissive, normal]:
		_listen(channel)

func _listen(channel: RefCounted) -> void:
	if not channel.changed.is_connected(changed.emit):
		channel.changed.connect(changed.emit)

func apply_to(mat: StandardMaterial3D) -> void:
	var resolve := func(texture_location: TextureLocation) -> Texture2D:
		if texture_location.source.is_empty():
			return null

		var images: Array = Assets.images(texture_location.source)
		return images[texture_location.image] if texture_location.image >= 0 and texture_location.image < images.size() else null

	var albedo_texture: Texture2D = resolve.call(albedo.texture) if albedo.kind == SceneChannel.Kind.TEXTURE else null
	var alpha := 1.0 - transparency
	var tint := albedo.rgb.constant
	mat.albedo_texture = albedo_texture
	mat.albedo_color = (Color(1, 1, 1, alpha) if albedo_texture != null
		else Color(tint.r, tint.g, tint.b, alpha))

	var metallic_texture: Texture2D = resolve.call(metallic.texture) if metallic.kind == SceneChannel.Kind.TEXTURE else null
	mat.metallic_texture = metallic_texture
	mat.metallic_texture_channel = metallic.scalar.channel
	mat.metallic = 1.0 if metallic_texture != null else metallic.scalar.constant

	var roughness_texture: Texture2D = resolve.call(roughness.texture) if roughness.kind == SceneChannel.Kind.TEXTURE else null
	mat.roughness_texture = roughness_texture
	mat.roughness_texture_channel = roughness.scalar.channel
	mat.roughness = 1.0 if roughness_texture != null else roughness.scalar.constant

	var emissive_texture: Texture2D = resolve.call(emissive.texture) if emissive.kind == SceneChannel.Kind.TEXTURE else null
	mat.emission_texture = emissive_texture
	mat.emission = Color.WHITE if emissive_texture != null else emissive.rgb.constant
	mat.emission_enabled = emissive_texture != null or emissive.rgb.constant != Color.BLACK

	var normal_texture: Texture2D = resolve.call(normal)
	mat.normal_texture = normal_texture
	mat.normal_enabled = normal_texture != null

	mat.transparency = (BaseMaterial3D.TRANSPARENCY_ALPHA if transparency > 0.0
		else BaseMaterial3D.TRANSPARENCY_DISABLED)
