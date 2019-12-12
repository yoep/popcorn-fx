package com.github.yoep.popcorn.config.properties;

import lombok.Data;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.context.annotation.Configuration;
import org.springframework.validation.annotation.Validated;

import javax.validation.Valid;
import javax.validation.constraints.NotNull;
import java.util.List;
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
     * The supported genres by the Popcorn API.
     * https://popcornofficial.docs.apiary.io/#reference/genres/page?console=1
     */
    @NotNull
    private List<String> genres;
    /**
     * The supported "sort by" by the Popcorn API.
     */
    @NotNull
    private List<String> sortBy;

    public ProviderProperties getProvider(String name) {
        return providers.entrySet().stream()
                .filter(e -> e.getKey().equalsIgnoreCase(name))
                .findFirst()
                .map(Map.Entry::getValue)
                .orElseThrow(() -> new ProviderNotFoundException(name));
    }
}
