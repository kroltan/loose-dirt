#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform ColorMaterial_color {
    vec4 Color;
};

layout(set = 1, binding = 1) uniform utexture2D ColorMaterial_texture;
layout(set = 1, binding = 2) uniform sampler ColorMaterial_texture_sampler;
// layout(set = 2, binding = 3) uniform TilemapContext {
//     float time;
//     vec2 texel_size;
// } Tilemap;

#define MAP_SAMPLER (usampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler))
#define MATERIAL_AT_OFFSET(x, y) (textureOffset(MAP_SAMPLER, v_Uv, ivec2((x), (y))).r)

void main() {
    vec4 color;
    uint material = texture(MAP_SAMPLER, v_Uv).x;
    switch (material) {
        case 0: color = vec4(1, 1, 1, 0.01); break;
        case 1:
            color = vec4(0.5, 0.5, 0.5, 1.0);

            // arbitrary sampling for semi-random looking results
            vec4 darken = vec4(0.04, 0.04, 0.04, 0);
            if (MATERIAL_AT_OFFSET(-1,  0) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 1,  0) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 0, -1) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 0,  1) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET(-2,  2) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 2, -2) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 2, -2) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET(-2,  2) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET(-3,  0) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 0, -3) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET( 0, -3) != 0) color -= darken;
            if (MATERIAL_AT_OFFSET(-3,  0) != 0) color -= darken;
            break;
        case 2:
            color = vec4(0.0, 0.1, 1.0, 0.8);

            // darken the water when space above is occupied
            vec4 shadow = vec4(0, 0.05, 0.05, 0);
            if (MATERIAL_AT_OFFSET(0, -1) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -2) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -3) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -4) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -5) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -6) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -7) == 0) color += shadow;
            if (MATERIAL_AT_OFFSET(0, -8) == 0) color += shadow;
            break;
        case 3:
            color = vec4(0.5, 0.5, 0.0, 1.0);

            // darken the sand when near water
            vec4 wetness = vec4(0.1, 0.1, 0.05, 0);
            if (MATERIAL_AT_OFFSET(-2,  0) == 2) color -= wetness;
            if (MATERIAL_AT_OFFSET(-1,  0) == 2) color -= wetness;
            if (MATERIAL_AT_OFFSET( 1,  0) == 2) color -= wetness;
            if (MATERIAL_AT_OFFSET( 2,  0) == 2) color -= wetness;
            if (MATERIAL_AT_OFFSET( 0, -1) == 2) color -= wetness;
            if (MATERIAL_AT_OFFSET( 0,  1) == 2) color -= wetness;
            break;
        default: color = vec4(1, 0, 1, 1); // unknown material
    }

    vec2 inner = v_Uv;

    //color = vec4(inner, 0, 1);
    o_Target = color;
}