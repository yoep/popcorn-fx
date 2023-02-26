package com.github.yoep.popcorn.backend.config.properties;

import lombok.Data;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.context.annotation.Configuration;
import org.springframework.validation.annotation.Validated;

import javax.validation.Valid;
import javax.validation.constraints.NotNull;

@Data
@Validated
@Configuration
@ConfigurationProperties("popcorn")
public class PopcornProperties {
    /**
     * The trakt properties for Popcorn FX.
     */
    @Valid
    @NotNull
    private TraktProperties trakt;
}
