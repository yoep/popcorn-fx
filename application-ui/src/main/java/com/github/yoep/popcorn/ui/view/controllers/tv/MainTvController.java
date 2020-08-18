package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.AbstractMainController;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class MainTvController extends AbstractMainController implements MainController {
    static final String TV_STYLESHEET = "/styles/tv.css";

    //region Constructors

    public MainTvController(ApplicationEventPublisher eventPublisher,
                            ViewLoader viewLoader,
                            ApplicationArguments arguments,
                            UrlService urlService,
                            SettingsService settingsService,
                            TaskExecutor taskExecutor) {
        super(eventPublisher, viewLoader, arguments, urlService, settingsService, taskExecutor);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeStylesheets();
    }

    private void initializeStylesheets() {
        rootPane.getStylesheets().add(TV_STYLESHEET);
    }

    //endregion
}
