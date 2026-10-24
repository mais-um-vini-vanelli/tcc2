@tool
class_name TransformSection extends Section

func bind(form: Form, node: SceneObject) -> void:
	form.bind(%PositionX, node, ^"position:x")
	form.bind(%PositionY, node, ^"position:y")
	form.bind(%PositionZ, node, ^"position:z")

	form.bind(%RotationDegreesX, node, ^"rotation_degrees:x")
	form.bind(%RotationDegreesY, node, ^"rotation_degrees:y")
	form.bind(%RotationDegreesZ, node, ^"rotation_degrees:z")

	form.bind(%ScaleX, node, ^"scale:x")
	form.bind(%ScaleY, node, ^"scale:y")
	form.bind(%ScaleZ, node, ^"scale:z")
