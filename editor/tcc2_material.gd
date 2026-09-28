class_name Tcc2Material extends Resource

@export var albedo: Tcc2ColorValue = Tcc2ColorValue.new():
	set(value):
		albedo = value if value != null else Tcc2ColorValue.new()
		_listen(albedo)
		emit_changed()

@export var metallic: Tcc2ScalarValue = Tcc2ScalarValue.new():
	set(value):
		metallic = value if value != null else Tcc2ScalarValue.new()
		_listen(metallic)
		emit_changed()

@export var roughness: Tcc2ScalarValue = Tcc2ScalarValue.new():
	set(value):
		roughness = value if value != null else Tcc2ScalarValue.new()
		_listen(roughness)
		emit_changed()

@export var emissive: Tcc2ColorValue = Tcc2ColorValue.new():
	set(value):
		emissive = value if value != null else Tcc2ColorValue.new()
		_listen(emissive)
		emit_changed()

@export var normal: Tcc2Texture = Tcc2Texture.new():
	set(value):
		normal = value if value != null else Tcc2Texture.new()
		_listen(normal)
		emit_changed()

# Whitted's model, orthogonal to the above.
@export_range(0.0, 1.0) var transparency: float = 0.0:
	set(value):
		transparency = value
		emit_changed()

@export_range(1.0, 3.0) var refractive_index: float = 1.0:
	set(value):
		refractive_index = value
		emit_changed()

# The setters don't run for the declarations' initial values.
func _init() -> void:
	albedo.constant = Color(0.8, 0.8, 0.8)
	roughness.constant = 0.5

	for channel in [albedo, metallic, roughness, emissive, normal]:
		_listen(channel)

func _listen(channel: Resource) -> void:
	if not channel.changed.is_connected(emit_changed):
		channel.changed.connect(emit_changed)

func apply_to(mat: StandardMaterial3D) -> void:
	var albedo_texture := albedo.resolve()
	var alpha := 1.0 - transparency
	mat.albedo_texture = albedo_texture
	mat.albedo_color = (Color(1, 1, 1, alpha) if albedo_texture != null
		else Color(albedo.constant.r, albedo.constant.g, albedo.constant.b, alpha))

	var metallic_texture := metallic.resolve()
	mat.metallic_texture = metallic_texture
	mat.metallic_texture_channel = metallic.channel
	mat.metallic = 1.0 if metallic_texture != null else metallic.constant

	var roughness_texture := roughness.resolve()
	mat.roughness_texture = roughness_texture
	mat.roughness_texture_channel = roughness.channel
	mat.roughness = 1.0 if roughness_texture != null else roughness.constant

	var emissive_texture := emissive.resolve()
	mat.emission_texture = emissive_texture
	mat.emission = Color.WHITE if emissive_texture != null else emissive.constant
	mat.emission_enabled = emissive_texture != null or emissive.constant != Color.BLACK

	var normal_texture := normal.resolve()
	mat.normal_texture = normal_texture
	mat.normal_enabled = normal_texture != null

	mat.transparency = (BaseMaterial3D.TRANSPARENCY_ALPHA if transparency > 0.0
		else BaseMaterial3D.TRANSPARENCY_DISABLED)
