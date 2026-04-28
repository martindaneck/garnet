#version 460 core

out float FragColor;

in vec2 TexCoords;

layout(std140, binding = 1) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
    float _padding2;
};

layout (binding=0) uniform sampler2D depth_texture;
layout (binding=1) uniform sampler2D normal_texture;
layout (binding=2) uniform sampler2D noise_texture;

uniform vec3 samples[64];

uniform float screen_width;
uniform float screen_height;

layout(std140, binding = 0) uniform Settings {
    float gamma;
    float exposure;
    int environment;
    int rendermode;
    float ssao_radius;
    float ssao_bias;
    float _padding3;
    float _padding4;
};


vec3 reconstruct_view_position(vec2 uv, float depth) {
    float z = depth * 2.0 - 1.0;
    vec4 clip_space_position = vec4(uv * 2.0 - 1.0, z, 1.0);
    vec4 view_space_position = inverse(projection) * clip_space_position;
    view_space_position /= view_space_position.w;
    return view_space_position.xyz;
}

void main() { // TODO 
    // debug
    //FragColor = samples[47].r; 
    //FragColor = texture(normal_texture, TexCoords).g;
    float depth = texture(depth_texture, TexCoords).r;
    if (depth >= 1.0) {
        FragColor = 1.0;
        return;
    }

    float radius = ssao_radius;
    float bias = ssao_bias;
    vec3 frag_pos = reconstruct_view_position(TexCoords, depth);
    vec3 normal = normalize(texture(normal_texture, TexCoords).rgb * 2.0 - 1.0);
    vec3 random_vec = normalize(texture(noise_texture, TexCoords * vec2(screen_width / 4.0, screen_height / 4.0)).xyz * 2.0 - 1.0);

    // tbn matrix
    vec3 T = normalize(random_vec - normal * dot(random_vec, normal));
    vec3 B = normalize(cross(normal, T));
    mat3 TBN = mat3(T, B, normal);
    
    float occlusion = 0.0;

    for (int i = 0; i < 64; ++i) {
        vec3 sample_pos = frag_pos + TBN * samples[i] * radius;

        vec4 offset = projection * vec4(sample_pos, 1.0);
        offset.xyz /= offset.w;
        offset.xyz = offset.xyz * 0.5 + 0.5;

        float sample_depth = texture(depth_texture, offset.xy).r;
        vec3 sample_view_pos = reconstruct_view_position(offset.xy, sample_depth);

        float range_check = smoothstep(0.0, 1.0, radius / abs(frag_pos.z - sample_view_pos.z));
        occlusion += (sample_view_pos.z > sample_pos.z + bias ? 1.0 : 0.0) * range_check;
    }

    FragColor = 1.0 - (occlusion / 64.0);

}