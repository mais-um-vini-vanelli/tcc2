class_name Tcc2TransformSection extends Tcc2Section

func bind(binder: Tcc2Binder, node: Tcc2Node) -> void:
	binder.bind_axis(%position_x, node, &"position", Vector3.AXIS_X)
	binder.bind_axis(%position_y, node, &"position", Vector3.AXIS_Y)
	binder.bind_axis(%position_z, node, &"position", Vector3.AXIS_Z)

	binder.bind_axis(%rotation_degrees_x, node, &"rotation_degrees", Vector3.AXIS_X)
	binder.bind_axis(%rotation_degrees_y, node, &"rotation_degrees", Vector3.AXIS_Y)
	binder.bind_axis(%rotation_degrees_z, node, &"rotation_degrees", Vector3.AXIS_Z)

	binder.bind_axis(%scale_x, node, &"scale", Vector3.AXIS_X)
	binder.bind_axis(%scale_y, node, &"scale", Vector3.AXIS_Y)
	binder.bind_axis(%scale_z, node, &"scale", Vector3.AXIS_Z)
