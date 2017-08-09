macro_rules! send_msg_macro{
    ($(($k_alias:ident,$k_function:ident,$k_type:ty)),*$(,)*) =>{
#[derive(Serialize, Deserialize, Default,Debug, Clone)]
#[serde(default)]
        pub struct SendMsg{
            $(
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
                pub $k_alias:Option<Option<$k_type>>),*
        }

        impl SendMsg{
            pub fn new()->SendMsg{
                SendMsg{
                    $( $k_alias:None),*
                }
            }
            pub fn serialize_send(a:SendMsg) -> Result<String, serde_json::Error> {
                serde_json::to_string(&a)
            }
            $( pub fn $k_function(&mut self,s:$k_type)->&mut Self{
                self.$k_alias = Some(Some(s));
                self
            })*
            
        }
    }
}
//this allows serde rename
macro_rules! receive_msg_macro{
    ( rename:{ $(($k_alias:ident,$k_function:ident,$k_type:ty,$k_rename:expr)),*$(,)*},
    else:{$(($e_alias:ident,$e_function:ident,$e_type:ty)),*$(,)*}
       ) =>{
#[derive(Serialize, Deserialize, Default,Debug, Clone)]
#[serde(default)]
        pub struct ReceivedMsg{
             $(
    #[serde(rename = $k_rename)]
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
                pub $k_alias:Option<Option<$k_type>>,)*
            $(
    #[serde(deserialize_with = "deserialize_optional_field")]
    #[serde(skip_serializing_if = "Option::is_none")]
                pub $e_alias:Option<Option<$e_type>>),*
        }

        impl ReceivedMsg{
            pub fn deserialize_receive(json: &str) -> Result<ReceivedMsg, serde_json::Error> {
                serde_json::from_str(json)
            }
            $( pub fn $e_function(&mut self,s:$e_type)->&mut Self{
                self.$e_alias = Some(Some(s));
                self
            })*
            $( pub fn $k_function(&mut self,s:$k_type)->&mut Self{
                self.$k_alias = Some(Some(s));
                self
            })* 
        }
    }
}
