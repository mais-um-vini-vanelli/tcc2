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

var emissive_strength: float = 1.0:
	set(value):
		emissive_strength = value
		changed.emit()

var normal: int = -1:
	set(value):
		normal = value
		changed.emit()

var normal_scale: float = 1.0:
	set(value):
		normal_scale = value
		changed.emit()

var transparency: float = 0.0:
	set(value):
		transparency = value
		changed.emit()

var refractive_index: float = 1.5:
	set(value):
		refractive_index = value
		changed.emit()

func _init() -> void:
	albedo.rgb.constant = Color(0.8, 0.8, 0.8)
	roughness.scalar.constant = 0.5

	for channel in [albedo, metallic, roughness, emissive]:
		_listen(channel)

func _listen(channel: RefCounted) -> void:
	if not channel.changed.is_connected(changed.emit):
		channel.changed.connect(changed.emit)

func apply_to(mat: StandardMaterial3D, assets: SceneAssets) -> void:
	var resolve := func(index: int) -> Texture2D:
		return assets.texture(index) if assets != null else null

	var albedo_texture: Texture2D = resolve.call(albedo.map) if albedo.mapped() else null
	var tint := Color.WHITE if albedo_texture != null else albedo.rgb.constant
	mat.albedo_texture = albedo_texture
	mat.albedo_color = Color(tint.r, tint.g, tint.b, 1.0 - transparency)

	var metallic_texture: Texture2D = resolve.call(metallic.map) if metallic.mapped() else null
	mat.metallic_texture = metallic_texture
	mat.metallic_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_RED
	mat.metallic = 1.0 if metallic_texture != null else metallic.scalar.constant

	var roughness_texture: Texture2D = resolve.call(roughness.map) if roughness.mapped() else null
	mat.roughness_texture = roughness_texture
	mat.roughness_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_RED
	mat.roughness = 1.0 if roughness_texture != null else roughness.scalar.constant

	var emissive_texture: Texture2D = resolve.call(emissive.map) if emissive.mapped() else null
	var emission := Color.WHITE if emissive_texture != null else emissive.rgb.constant
	mat.emission_texture = emissive_texture
	mat.emission = emission
	mat.emission_energy_multiplier = emissive_strength
	mat.emission_enabled = (emissive_strength > 0.0
		and (emissive_texture != null or emission != Color.BLACK))

	var normal_texture: Texture2D = resolve.call(normal)
	mat.normal_texture = normal_texture
	mat.normal_enabled = normal_texture != null
	mat.normal_scale = normal_scale

	mat.transparency = (BaseMaterial3D.TRANSPARENCY_ALPHA if transparency > 0.0
		else BaseMaterial3D.TRANSPARENCY_DISABLED)
