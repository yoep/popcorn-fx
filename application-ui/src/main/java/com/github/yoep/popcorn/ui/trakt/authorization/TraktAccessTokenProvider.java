package com.github.yoep.popcorn.ui.trakt.authorization;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.OAuth2AccessTokenWrapper;
import com.github.yoep.popcorn.backend.settings.models.TraktSettings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpHeaders;
import org.springframework.security.access.AccessDeniedException;
import org.springframework.security.oauth2.client.resource.OAuth2ProtectedResourceDetails;
import org.springframework.security.oauth2.client.resource.UserApprovalRequiredException;
import org.springframework.security.oauth2.client.resource.UserRedirectRequiredException;
import org.springframework.security.oauth2.client.token.AccessTokenProvider;
import org.springframework.security.oauth2.client.token.AccessTokenRequest;
import org.springframework.security.oauth2.client.token.DefaultAccessTokenRequest;
import org.springframework.security.oauth2.client.token.grant.code.AuthorizationCodeAccessTokenProvider;
import org.springframework.security.oauth2.common.DefaultOAuth2AccessToken;
import org.springframework.security.oauth2.common.OAuth2AccessToken;
import org.springframework.stereotype.Component;
import org.springframework.util.LinkedMultiValueMap;
import org.springframework.util.MultiValueMap;

import java.time.LocalDateTime;
import java.time.ZoneId;
import java.util.Optional;
import java.util.concurrent.ExecutionException;

@Slf4j
@Component
@RequiredArgsConstructor
public class TraktAccessTokenProvider extends AuthorizationCodeAccessTokenProvider implements AccessTokenProvider {
    private final PopcornProperties properties;
    private final SettingsService settingsService;
    private final AuthorizationService authorizationService;

    @Override
    public OAuth2AccessToken obtainAccessToken(OAuth2ProtectedResourceDetails details, AccessTokenRequest parameters)
            throws UserRedirectRequiredException, UserApprovalRequiredException, AccessDeniedException {
        try {
            var optionalAccessToken = getSettings().getAccessToken();

            return optionalAccessToken
                    .map(accessTokenWrapper -> resolveAccessToken(details, accessTokenWrapper))
                    .orElseGet(() -> super.obtainAccessToken(details, parameters));
        } catch (RefreshTokenMissingException ex) {
            log.error(ex.getMessage(), ex);
            return handleRefreshTokenMissing(details, parameters);
        } catch (UserRedirectRequiredException ex) {
            return startUserRedirect(details, ex);
        }
    }

    private OAuth2AccessToken startUserRedirect(OAuth2ProtectedResourceDetails details, UserRedirectRequiredException ex) {
        String accessToken;

        try {
            accessToken = authorizationService.startAuthorization(ex, properties.getTrakt().getClient().getPreEstablishedRedirectUri()).get();
        } catch (InterruptedException | ExecutionException exc) {
            throw new AccessTokenException(exc.getMessage(), exc);
        }
        return retrieveAccessToken(details, accessToken, Optional.ofNullable(ex.getStateToPreserve())
                .map(Object::toString)
                .orElse(null));
    }

    private OAuth2AccessToken resolveAccessToken(OAuth2ProtectedResourceDetails details, OAuth2AccessTokenWrapper accessTokenWrapper) {
        if (!accessTokenWrapper.isExpired()) {
            return accessTokenWrapper.getToken();
        } else if (accessTokenWrapper.getToken().getRefreshToken() != null) {
            return retrieveRefreshToken(details, accessTokenWrapper.getToken());
        } else {
            throw new RefreshTokenMissingException();
        }
    }

    private OAuth2AccessToken retrieveAccessToken(OAuth2ProtectedResourceDetails resource, String authorizationCode, String redirectUri) {
        final AccessTokenRequest request = new DefaultAccessTokenRequest();

        request.setAuthorizationCode(authorizationCode);
        request.setPreservedState(redirectUri);

        OAuth2AccessToken oAuth2AccessToken = retrieveToken(request, resource, getParametersForTokenRequest(authorizationCode, redirectUri), new HttpHeaders());
        saveAccessToken(oAuth2AccessToken);
        return oAuth2AccessToken;
    }

    private OAuth2AccessToken retrieveRefreshToken(OAuth2ProtectedResourceDetails resource, OAuth2AccessToken accessToken) {
        var request = new DefaultAccessTokenRequest();
        var oAuth2AccessToken
                = (DefaultOAuth2AccessToken) retrieveToken(request, resource, getParametersForRefreshTokenRequest(accessToken), new HttpHeaders());
        oAuth2AccessToken.setRefreshToken(accessToken.getRefreshToken());
        saveAccessToken(oAuth2AccessToken);

        return oAuth2AccessToken;
    }

    private OAuth2AccessToken handleRefreshTokenMissing(OAuth2ProtectedResourceDetails details, AccessTokenRequest parameters) {
        try {
            return super.obtainAccessToken(details, parameters);
        } catch (UserRedirectRequiredException userRedirect) {
            return startUserRedirect(details, userRedirect);
        }
    }

    private MultiValueMap<String, String> getParametersForTokenRequest(String authorizationCode,
                                                                       String redirectUri) {
        MultiValueMap<String, String> form = new LinkedMultiValueMap<>();
        form.set("grant_type", "authorization_code");
        form.set("code", authorizationCode);
        form.set("redirect_uri", redirectUri);
        return form;
    }

    private MultiValueMap<String, String> getParametersForRefreshTokenRequest(OAuth2AccessToken accessToken) {
        MultiValueMap<String, String> form = new LinkedMultiValueMap<>();
        form.set("grant_type", "refresh_token");
        form.set("refresh_token", accessToken.getRefreshToken().getValue());
        return form;
    }

    private TraktSettings getSettings() {
        return settingsService.getSettings().getTraktSettings();
    }

    private void saveAccessToken(OAuth2AccessToken oAuth2AccessToken) {
        if (oAuth2AccessToken == null)
            return;

        getSettings().setAccessToken(OAuth2AccessTokenWrapper.builder()
                .expireDate(LocalDateTime.ofInstant(oAuth2AccessToken.getExpiration().toInstant(), ZoneId.systemDefault()))
                .token(oAuth2AccessToken)
                .build());
    }
}
