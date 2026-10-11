shader_type canvas_item;

// White where a selected object is, black everywhere else.
uniform sampler2D mask : filter_linear;
uniform vec2 mask_size = vec2(1.0);
uniform float width = 2.0;
uniform vec4 line_color : source_color = vec4(1.0);

void fragment() {
	// Half the kernel, in pixels. 1 extra pixel because the edges get antialiased
	int radius = int(width) + 1;
	float radiusf = float(radius);
	float near = 0.0;

	for (int y = -radius; y <= radius; y++) {
		for (int x = -radius; x <= radius; x++) {
			vec2 tap = vec2(float(x), float(y));

			// if tap.x² + tap.y² > radius², we are outside the disc
			if (dot(tap, tap) > radiusf * radiusf) {
				continue;
			}

			near = max(near, texture(mask, UV + tap / mask_size).r);
		}
	}

	float inside = texture(mask, UV).r;
	COLOR = vec4(line_color.rgb, line_color.a * near * (1.0 - inside));
}
