package com.github.yoep.popcorn.ui.trakt.config;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.trakt.authorization.TraktAccessTokenProvider;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.http.HttpHeaders;
import org.springframework.security.oauth2.client.DefaultOAuth2ClientContext;
import org.springframework.security.oauth2.client.OAuth2RestOperations;
import org.springframework.security.oauth2.client.OAuth2RestTemplate;
import org.springframework.security.oauth2.client.token.AccessTokenProviderChain;
import org.springframework.security.oauth2.client.token.grant.client.ClientCredentialsAccessTokenProvider;
import org.springframework.security.oauth2.client.token.grant.code.AuthorizationCodeAccessTokenProvider;
import org.springframework.security.oauth2.client.token.grant.implicit.ImplicitAccessTokenProvider;
import org.springframework.security.oauth2.client.token.grant.password.ResourceOwnerPasswordAccessTokenProvider;

import java.util.Collections;

import static java.util.Arrays.asList;

@Configuration
public class TraktConfig {
    @Bean
    public OAuth2RestOperations traktTemplate(TraktAccessTokenProvider accessTokenProvider, PopcornProperties properties) {
        var oAuth2RestTemplate = new OAuth2RestTemplate(properties.getTrakt().getClient(), new DefaultOAuth2ClientContext());

        oAuth2RestTemplate.setAccessTokenProvider(new AccessTokenProviderChain(asList(
                accessTokenProvider,
                new AuthorizationCodeAccessTokenProvider(),
                new ImplicitAccessTokenProvider(),
                new ResourceOwnerPasswordAccessTokenProvider(),
                new ClientCredentialsAccessTokenProvider())));
        oAuth2RestTemplate.setInterceptors(Collections.singletonList((request, body, execution) -> {
            HttpHeaders headers = request.getHeaders();

            headers.add("trakt-api-version", "2");
            headers.add("trakt-api-key", properties.getTrakt().getClient().getClientId());

            return execution.execute(request, body);
        }));

        return oAuth2RestTemplate;
    }
}
