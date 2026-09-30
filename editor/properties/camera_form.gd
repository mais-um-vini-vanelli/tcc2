class_name Tcc2CameraForm extends Tcc2Form

const TITLES := ["Perspective", "Orthographic"]

func _build() -> void:
	var camera := node as Tcc2Camera
	var model: Object = camera.model()
	var orthographic := camera.kind == Tcc2Camera.Kind.ORTHOGRAPHIC

	%Camera.title = TITLES[camera.kind]

	%Camera.show_row(%fov, not orthographic)
	%Camera.show_row(%height, orthographic)

	if orthographic:
		bind(%height, model, &"height")
	else:
		bind(%fov, model, &"fov")

	bind(%near, model, &"near")
	bind(%far, model, &"far")

	%Transform.bind(binder, camera)
