class_name GeometryForm extends Form

const TITLES := ["Sphere", "Cube", "Mesh", "Triangle"]

func _build() -> void:
	var geometry := node as SceneGeometry
	var model: Object = geometry.model()
	var kind := geometry.kind
	var from_file := kind == SceneGeometry.Kind.MESH

	%Geometry.title = TITLES[kind]

	%RadiusRow.visible = kind == SceneGeometry.Kind.SPHERE
	%HalfEdgeRow.visible = kind == SceneGeometry.Kind.CUBE

	for row in [%AXRow, %AYRow, %AZRow, %BXRow, %BYRow, %BZRow,
			%CXRow, %CYRow, %CZRow]:
		row.visible = kind == SceneGeometry.Kind.TRIANGLE

	%SourceRow.visible = from_file
	%IndexRow.visible = from_file

	match kind:
		SceneGeometry.Kind.SPHERE:
			bind(%Radius, model, ^"radius")
		SceneGeometry.Kind.CUBE:
			bind(%HalfEdge, model, ^"half_edge")
		SceneGeometry.Kind.TRIANGLE:
			bind(%AX, model, ^"a:x")
			bind(%AY, model, ^"a:y")
			bind(%AZ, model, ^"a:z")
			bind(%BX, model, ^"b:x")
			bind(%BY, model, ^"b:y")
			bind(%BZ, model, ^"b:z")
			bind(%CX, model, ^"c:x")
			bind(%CY, model, ^"c:y")
			bind(%CZ, model, ^"c:z")
		SceneGeometry.Kind.MESH:
			asset_picker(%SourceRow, %IndexRow, model, ^"index", SceneAssets.Kind.MESH)

	%Transform.bind(self, geometry)
	%Material.bind(self, geometry.material)
