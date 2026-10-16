class_name Tcc2LightForm extends Tcc2Form

const TITLES := ["Point", "Directional", "Spot", "Environment"]

func _build() -> void:
	var light := node as Tcc2Light
	var model: Object = light.model()
	var spot := light.kind == Tcc2Light.Kind.SPOT

	%Light.title = TITLES[light.kind]

	%Light.show_row(%inner_angle, spot)
	%Light.show_row(%outer_angle, spot)

	bind(%color, model, &"color")
	bind(%intensity, model, &"intensity")

	if spot:
		bind(%inner_angle, model, &"inner_angle")
		bind(%outer_angle, model, &"outer_angle")

	%Transform.bind(binder, light)
