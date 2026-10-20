class_name LightForm extends Form

const TITLES := ["Point", "Directional", "Spot", "Environment"]

func _build() -> void:
	var light := node as SceneLight
	var model: Object = light.model()
	var spot := light.kind == SceneLight.Kind.SPOT

	%Light.title = TITLES[light.kind]

	%Light.show_row(%inner_angle, spot)
	%Light.show_row(%outer_angle, spot)

	bind(%color, model, &"color")
	bind(%intensity, model, &"intensity")

	if spot:
		bind(%inner_angle, model, &"inner_angle")
		bind(%outer_angle, model, &"outer_angle")

	%Transform.bind(binder, light)
