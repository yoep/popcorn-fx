package com.github.yoep.popcorn.backend.config.properties;

import lombok.AllArgsConstructor;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import org.springframework.boot.context.properties.ConstructorBinding;
import org.springframework.security.oauth2.client.token.grant.code.AuthorizationCodeResourceDetails;

import javax.validation.constraints.NotNull;
import java.net.URI;

@Getter
@ConstructorBinding
@AllArgsConstructor
@ToString
@EqualsAndHashCode
public class TraktProperties {
    /**
     * The base url of the trakt.tv API.
     */
    @NotNull
    private final URI url;
    /**
     * The client details of the trakt api.
     */
    @NotNull
    private final AuthorizationCodeResourceDetails client;
}
