
use database::current::read::GetDbReadCommandsCommon;
use model::{GetReportList, ReportIteratorQueryInternal};

use crate::{
    db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminReport);

impl ReadCommandsCommonAdminReport<'_> {
    pub async fn get_waiting_report_list(
        &self,
    ) -> Result<GetReportList, DataError> {
        let components = self.config().components();
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .report()
                .get_reports_page(components)
        })
        .await
        .into_error()
    }

    pub async fn get_report_iterator_page(
        &self,
        query: ReportIteratorQueryInternal,
    ) -> Result<GetReportList, DataError> {
        let components = self.config().components();
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .report()
                .get_report_iterator_page(query, components)
        })
        .await
        .into_error()
    }
}
