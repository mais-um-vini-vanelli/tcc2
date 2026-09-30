class_name Tcc2GeometryForm extends Tcc2Form

const TITLES := ["Sphere", "Cube", "Mesh"]

func _build() -> void:
	var geometry := node as Tcc2Geometry
	var model: Object = geometry.model()
	var kind := geometry.kind
	var from_file := kind == Tcc2Geometry.Kind.MESH

	%Geometry.title = TITLES[kind]

	%Geometry.show_row(%radius, kind == Tcc2Geometry.Kind.SPHERE)
	%Geometry.show_row(%half_edge, kind == Tcc2Geometry.Kind.CUBE)
	%Geometry.show_row(%source, from_file)
	%Geometry.show_row(%mesh, from_file)
	%Geometry.show_row(%surface, from_file)

	match kind:
		Tcc2Geometry.Kind.SPHERE:
			bind(%radius, model, &"radius")
		Tcc2Geometry.Kind.CUBE:
			bind(%half_edge, model, &"half_edge")
		Tcc2Geometry.Kind.MESH:
			bind(%source, model, &"source")
			bind(%mesh, model, &"mesh")
			bind(%surface, model, &"surface")

	bind(%cast_shadows, geometry, &"cast_shadows")

	%Transform.bind(binder, geometry)
	%Material.bind(binder, geometry.material)
