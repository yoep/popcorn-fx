package com.github.yoep.popcorn.config.properties;

import lombok.Data;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.context.annotation.Configuration;
import org.springframework.validation.annotation.Validated;

import javax.validation.Valid;
import javax.validation.constraints.NotNull;
import java.util.Map;

@Data
@Validated
@Configuration
@ConfigurationProperties("popcorn")
public class PopcornProperties {
    /**
     * The providers for the available categories in Popcorn Time.
     */
    @Valid
    @NotNull
    private Map<String, ProviderProperties> providers;

    /**
     * The subtitle properties of Popcorn Time.
     */
    @Valid
    @NotNull
    private SubtitleProperties subtitle;

    /**
     * The trakt properties for Popcorn Time.
     */
    @Valid
    @NotNull
    private TraktProperties trakt;

    /**
     * Get the provider with the given name.
     *
     * @param name The name of the provider to retrieve.
     * @return Returns the provider if found.
     * @throws ProviderNotFoundException Is thrown when the given provider name couldn't be found.
     */
    public ProviderProperties getProvider(String name) {
        return providers.entrySet().stream()
                .filter(e -> e.getKey().equalsIgnoreCase(name))
                .findFirst()
                .map(Map.Entry::getValue)
                .orElseThrow(() -> new ProviderNotFoundException(name));
    }
}
