class_name Tcc2GeometryForm extends Tcc2Form

const TITLES := ["Sphere", "Cube", "Mesh", "Triangle"]

func _build() -> void:
	var geometry := node as Tcc2Geometry
	var model: Object = geometry.model()
	var kind := geometry.kind
	var from_file := kind == Tcc2Geometry.Kind.MESH

	%Geometry.title = TITLES[kind]

	%Geometry.show_row(%radius, kind == Tcc2Geometry.Kind.SPHERE)
	%Geometry.show_row(%half_edge, kind == Tcc2Geometry.Kind.CUBE)

	for field in [%a_x, %a_y, %a_z, %b_x, %b_y, %b_z, %c_x, %c_y, %c_z]:
		%Geometry.show_row(field, kind == Tcc2Geometry.Kind.TRIANGLE)

	%Geometry.show_row(%source, from_file)
	%Geometry.show_row(%mesh, from_file)
	%Geometry.show_row(%surface, from_file)

	match kind:
		Tcc2Geometry.Kind.SPHERE:
			bind(%radius, model, &"radius")
		Tcc2Geometry.Kind.CUBE:
			bind(%half_edge, model, &"half_edge")
		Tcc2Geometry.Kind.TRIANGLE:
			binder.bind_axis(%a_x, model, &"a", Vector3.AXIS_X)
			binder.bind_axis(%a_y, model, &"a", Vector3.AXIS_Y)
			binder.bind_axis(%a_z, model, &"a", Vector3.AXIS_Z)
			binder.bind_axis(%b_x, model, &"b", Vector3.AXIS_X)
			binder.bind_axis(%b_y, model, &"b", Vector3.AXIS_Y)
			binder.bind_axis(%b_z, model, &"b", Vector3.AXIS_Z)
			binder.bind_axis(%c_x, model, &"c", Vector3.AXIS_X)
			binder.bind_axis(%c_y, model, &"c", Vector3.AXIS_Y)
			binder.bind_axis(%c_z, model, &"c", Vector3.AXIS_Z)
		Tcc2Geometry.Kind.MESH:
			bind(%source, model, &"source")
			bind(%mesh, model, &"mesh")
			bind(%surface, model, &"surface")

	bind(%cast_shadows, geometry, &"cast_shadows")

	%Transform.bind(binder, geometry)
	%Material.bind(binder, geometry.material)
