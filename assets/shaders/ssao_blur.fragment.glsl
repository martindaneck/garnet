#version 460 core
out float FragColor;

in vec2 TexCoords;

uniform sampler2D ao_texture;
uniform sampler2D depth_texture;
uniform vec2 texelSize;

void main()
{
    float centerDepth = texture(depth_texture, TexCoords).r;

    float sum = 0.0;
    float weightSum = 0.0;

    for (int x = -2; x <= 2; x++)
    for (int y = -2; y <= 2; y++)
    {
        vec2 offset = vec2(x, y) * texelSize;
        float ao = texture(ao_texture, TexCoords + offset).r;
        float depth = texture(depth_texture, TexCoords + offset).r;

        float weight = abs(depth - centerDepth) < 0.02 ? 1.0 : 0.0;
        sum += ao * weight;
        weightSum += weight;
    }

    FragColor = sum / max(weightSum, 1.0);
}
