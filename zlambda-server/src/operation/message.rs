use actix::Message;
use zlambda_common::operation::OperationRequest;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HandleOperationRequestMessage {
    operation_request: OperationRequest,
}

impl From<HandleOperationRequestMessage> for (OperationRequest,) {
    fn from(message: HandleOperationRequestMessage) -> Self {
        (message.operation_request,)
    }
}

impl Message for HandleOperationRequestMessage {
    type Result = ();
}

impl HandleOperationRequestMessage {
    pub fn new(operation_request: OperationRequest) -> Self {
        Self {
            operation_request,
        }
    }

    pub fn operation_request(&self) -> &OperationRequest {
        &self.operation_request
    }

    pub fn operation_request_mut(&mut self) -> &mut OperationRequest {
        &mut self.operation_request
    }

    pub fn set_operation_request(&mut self, operation_request: OperationRequest) {
        self.operation_request = operation_request
    }
}
