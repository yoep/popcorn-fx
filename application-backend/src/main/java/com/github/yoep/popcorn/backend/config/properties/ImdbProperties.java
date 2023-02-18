package com.github.yoep.popcorn.backend.config.properties;

import lombok.AllArgsConstructor;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.boot.context.properties.ConstructorBinding;

import javax.validation.constraints.NotNull;
import java.net.URI;

@Getter
@ConstructorBinding
@AllArgsConstructor
@ToString
@EqualsAndHashCode
public class ImdbProperties {
    /**
     * The url to IMDB website.
     */
    @NotNull
    private final URI url;
}
