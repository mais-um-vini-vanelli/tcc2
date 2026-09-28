class_name Tcc2PointLight extends Tcc2Light

func _light_preview() -> Light3D:
	return OmniLight3D.new()
