class_name LightForm extends Form

const TITLES := ["Point", "Directional", "Spot", "Environment"]

func _build() -> void:
	var light := node as SceneLight
	var model: Object = light.model()
	var spot := light.kind == SceneLight.Kind.SPOT

	%Light.title = TITLES[light.kind]

	%InnerAngleRow.visible = spot
	%OuterAngleRow.visible = spot

	bind(%Color, model, ^"color")
	bind(%Intensity, model, ^"intensity")

	if spot:
		bind(%InnerAngle, model, ^"inner_angle")
		bind(%OuterAngle, model, ^"outer_angle")

	%Transform.bind(self, light)
